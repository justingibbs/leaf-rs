//! Google Gemini provider implementation

use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace};

use super::{
    ChatChunk, ChatMessage, ChatResponse, ChatRole, ChatStream, ContentBlockType, LlmProvider,
    ProviderConfig, ProviderType, StopReason, ToolCallRequest, ToolDefinition, Usage,
};
use crate::error::{AgentError, AgentResult};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Google Gemini provider
pub struct GoogleProvider {
    client: Client,
    config: ProviderConfig,
}

impl GoogleProvider {
    /// Create a new Google provider
    pub fn new(config: ProviderConfig) -> AgentResult<Self> {
        config.api_key.as_ref().ok_or_else(|| {
            AgentError::ProviderNotConfigured("Google API key required".to_string())
        })?;

        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                headers
            })
            .build()?;

        Ok(Self { client, config })
    }

    fn api_url(&self, stream: bool) -> String {
        let base = self
            .config
            .base_url
            .as_deref()
            .unwrap_or(GEMINI_API_BASE);
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        let api_key = self.config.api_key.as_deref().unwrap_or("");
        format!(
            "{}/{}:{}?key={}{}",
            base,
            self.config.model,
            method,
            api_key,
            if stream { "&alt=sse" } else { "" }
        )
    }

    fn build_request(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<GeminiRequest> {
        // Extract system instruction if present
        let system_instruction = messages
            .iter()
            .find(|m| m.role == ChatRole::System)
            .map(|m| GeminiContent {
                role: None,
                parts: vec![GeminiPart::Text { text: m.content.clone() }],
            });

        // Convert messages to Gemini format (skip system messages)
        let contents: Vec<GeminiContent> = messages
            .iter()
            .filter(|m| m.role != ChatRole::System)
            .map(|m| self.to_gemini_content(m))
            .collect::<AgentResult<Vec<_>>>()?;

        // Convert tools to Gemini format
        let tools_config = tools.map(|t| {
            vec![GeminiTool {
                function_declarations: t
                    .iter()
                    .map(|tool| GeminiFunctionDeclaration {
                        name: tool.name.clone(),
                        description: tool.description.clone(),
                        parameters: tool.input_schema.clone(),
                    })
                    .collect(),
            }]
        });

        Ok(GeminiRequest {
            contents,
            system_instruction,
            tools: tools_config,
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(self.config.temperature),
                max_output_tokens: Some(self.config.max_tokens),
                candidate_count: Some(1),
            }),
        })
    }

    fn to_gemini_content(&self, msg: &ChatMessage) -> AgentResult<GeminiContent> {
        // Gemini uses "function" role for tool/function responses
        let role = if msg.tool_result.is_some() {
            "function"
        } else {
            match msg.role {
                ChatRole::User => "user",
                ChatRole::Assistant => "model",
                ChatRole::System => "user", // Handled separately
            }
        };

        let mut parts: Vec<GeminiPart> = Vec::new();

        // Add tool result if present (function response)
        if let Some(ref tool_result) = msg.tool_result {
            parts.push(GeminiPart::FunctionResponse {
                function_response: GeminiFunctionResponse {
                    name: tool_result.tool_use_id.clone(),
                    response: tool_result.content.clone(),
                },
            });
        }

        // Add text content if present and non-empty
        if !msg.content.is_empty() {
            parts.push(GeminiPart::Text {
                text: msg.content.clone(),
            });
        }

        // Add tool calls if present (function calls from assistant)
        for tool_call in &msg.tool_calls {
            parts.push(GeminiPart::FunctionCall {
                function_call: GeminiFunctionCall {
                    name: tool_call.name.clone(),
                    args: tool_call.arguments.clone(),
                },
            });
        }

        // Ensure we have at least one part
        if parts.is_empty() {
            parts.push(GeminiPart::Text {
                text: String::new(),
            });
        }

        Ok(GeminiContent {
            role: Some(role.to_string()),
            parts,
        })
    }

    fn parse_response(&self, response: GeminiResponse) -> AgentResult<ChatResponse> {
        let candidate = response
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| AgentError::ProviderError("No candidates in response".to_string()))?;

        let mut text_content = String::new();
        let mut tool_calls = Vec::new();

        for part in candidate.content.parts {
            match part {
                GeminiPart::Text { text } => {
                    text_content.push_str(&text);
                }
                GeminiPart::FunctionCall { function_call } => {
                    tool_calls.push(ToolCallRequest {
                        id: function_call.name.clone(), // Gemini doesn't use separate IDs
                        name: function_call.name,
                        arguments: function_call.args,
                    });
                }
                _ => {}
            }
        }

        let stop_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => {
                if tool_calls.is_empty() {
                    StopReason::EndTurn
                } else {
                    StopReason::ToolUse
                }
            }
            Some("MAX_TOKENS") => StopReason::MaxTokens,
            Some("SAFETY") | Some("RECITATION") | Some("OTHER") => StopReason::Unknown,
            _ => {
                if !tool_calls.is_empty() {
                    StopReason::ToolUse
                } else {
                    StopReason::EndTurn
                }
            }
        };

        let usage = response.usage_metadata.map(|u| Usage {
            input_tokens: u.prompt_token_count.unwrap_or(0),
            output_tokens: u.candidates_token_count.unwrap_or(0),
        });

        Ok(ChatResponse {
            content: text_content,
            tool_calls,
            stop_reason,
            usage,
        })
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatResponse> {
        let request = self.build_request(messages, tools)?;
        let url = self.api_url(false);

        debug!("Sending request to Gemini API: model={}", self.config.model);
        trace!("Request body: {:?}", serde_json::to_string(&request));

        let response = self.client.post(&url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Gemini API error: {} - {}", status, error_text);
            return Err(AgentError::ProviderError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        debug!(
            "Received response: candidates={}",
            gemini_response.candidates.len()
        );

        self.parse_response(gemini_response)
    }

    async fn stream_chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatStream> {
        let request = self.build_request(messages, tools)?;
        let url = self.api_url(true);

        debug!(
            "Starting streaming request to Gemini API: model={}",
            self.config.model
        );

        let response = self.client.post(&url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Gemini API error: {} - {}", status, error_text);
            return Err(AgentError::ProviderError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        // Create SSE stream
        let byte_stream = response.bytes_stream();
        let stream = parse_gemini_sse_stream(byte_stream);

        Ok(Box::pin(stream))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Google
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Parse SSE stream from Gemini API
fn parse_gemini_sse_stream<S>(byte_stream: S) -> impl Stream<Item = AgentResult<ChatChunk>>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + Unpin + 'static,
{
    use std::collections::VecDeque;

    let buffer = String::new();
    let content_index: u32 = 0;
    let started_text = false;
    let pending_tool_calls: Vec<ToolCallRequest> = Vec::new();
    let queued_chunks: VecDeque<ChatChunk> = VecDeque::new();
    let done = false;

    futures::stream::unfold(
        (byte_stream, buffer, content_index, started_text, pending_tool_calls, queued_chunks, done),
        |(mut stream, mut buffer, mut content_index, mut started_text, mut pending_tool_calls, mut queued_chunks, mut done)| async move {
            use futures::StreamExt;

            // If the stream is done, stop producing items
            if done {
                return None;
            }

            // Yield any queued chunks from a previous SSE event first
            if let Some(chunk) = queued_chunks.pop_front() {
                return Some((
                    Ok(chunk),
                    (stream, buffer, content_index, started_text, pending_tool_calls, queued_chunks, done),
                ));
            }

            loop {
                // Check if we have a complete line in the buffer
                if let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    // Skip empty lines and comments
                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }

                    // Parse SSE data line
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            // Emit message stop
                            let stop_reason = if pending_tool_calls.is_empty() {
                                StopReason::EndTurn
                            } else {
                                StopReason::ToolUse
                            };
                            done = true;
                            return Some((
                                Ok(ChatChunk::MessageStop { stop_reason }),
                                (stream, buffer, content_index, started_text, pending_tool_calls, queued_chunks, done),
                            ));
                        }

                        match serde_json::from_str::<GeminiResponse>(data) {
                            Ok(response) => {
                                if let Some(chunks) = process_gemini_stream_response(
                                    response,
                                    &mut content_index,
                                    &mut started_text,
                                    &mut pending_tool_calls,
                                ) {
                                    let mut iter = chunks.into_iter();
                                    if let Some(first) = iter.next() {
                                        // Queue remaining chunks for subsequent calls
                                        queued_chunks.extend(iter);
                                        return Some((
                                            Ok(first),
                                            (stream, buffer, content_index, started_text, pending_tool_calls, queued_chunks, done),
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                trace!("Failed to parse Gemini SSE event: {} - {}", e, data);
                            }
                        }
                    }
                    continue;
                }

                // Need more data from the stream
                match stream.next().await {
                    Some(Ok(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                    }
                    Some(Err(e)) => {
                        return Some((
                            Err(AgentError::HttpError(e)),
                            (stream, buffer, content_index, started_text, pending_tool_calls, queued_chunks, done),
                        ));
                    }
                    None => {
                        // Stream ended - emit final stop if we haven't already
                        if started_text || !pending_tool_calls.is_empty() {
                            let stop_reason = if pending_tool_calls.is_empty() {
                                StopReason::EndTurn
                            } else {
                                StopReason::ToolUse
                            };
                            // Mark done so we don't loop back and emit another MessageStop
                            done = true;
                            return Some((
                                Ok(ChatChunk::MessageStop { stop_reason }),
                                (stream, buffer, content_index, started_text, pending_tool_calls, queued_chunks, done),
                            ));
                        }
                        return None;
                    }
                }
            }
        },
    )
}

fn process_gemini_stream_response(
    response: GeminiResponse,
    content_index: &mut u32,
    started_text: &mut bool,
    pending_tool_calls: &mut Vec<ToolCallRequest>,
) -> Option<Vec<ChatChunk>> {
    let mut chunks = Vec::new();

    for candidate in response.candidates {
        for part in candidate.content.parts {
            match part {
                GeminiPart::Text { text } => {
                    if !*started_text {
                        // Emit content block start
                        chunks.push(ChatChunk::ContentBlockStart {
                            index: *content_index,
                            content_type: ContentBlockType::Text,
                        });
                        *started_text = true;
                    }
                    // Emit text delta
                    chunks.push(ChatChunk::TextDelta {
                        index: *content_index,
                        text,
                    });
                }
                GeminiPart::FunctionCall { function_call } => {
                    // Emit content block start for tool
                    *content_index += 1;
                    let tool_id = function_call.name.clone();
                    let tool_name = function_call.name.clone();

                    chunks.push(ChatChunk::ContentBlockStart {
                        index: *content_index,
                        content_type: ContentBlockType::ToolUse {
                            id: tool_id.clone(),
                            name: tool_name.clone(),
                        },
                    });

                    // Emit tool use delta with the full JSON
                    let args_json = serde_json::to_string(&function_call.args).unwrap_or_default();
                    chunks.push(ChatChunk::ToolUseDelta {
                        index: *content_index,
                        partial_json: args_json,
                    });

                    // Emit content block stop
                    chunks.push(ChatChunk::ContentBlockStop {
                        index: *content_index,
                    });

                    pending_tool_calls.push(ToolCallRequest {
                        id: tool_id,
                        name: tool_name,
                        arguments: function_call.args,
                    });
                }
                _ => {}
            }
        }

        // Check finish reason
        if let Some(reason) = candidate.finish_reason {
            let stop_reason = match reason.as_str() {
                "STOP" => {
                    if pending_tool_calls.is_empty() {
                        StopReason::EndTurn
                    } else {
                        StopReason::ToolUse
                    }
                }
                "MAX_TOKENS" => StopReason::MaxTokens,
                _ => StopReason::Unknown,
            };

            // Close text block if open
            if *started_text {
                chunks.push(ChatChunk::ContentBlockStop {
                    index: 0,
                });
            }

            chunks.push(ChatChunk::MessageStop { stop_reason });
        }
    }

    // Add usage if available
    if let Some(usage) = response.usage_metadata {
        chunks.push(ChatChunk::Usage(Usage {
            input_tokens: usage.prompt_token_count.unwrap_or(0),
            output_tokens: usage.candidates_token_count.unwrap_or(0),
        }));
    }

    if chunks.is_empty() {
        None
    } else {
        Some(chunks)
    }
}

// Gemini API types

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFunctionResponse,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiTool {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request() {
        let config = ProviderConfig {
            provider: ProviderType::Google,
            api_key: Some("test-key".to_string()),
            model: "gemini-1.5-pro".to_string(),
            ..Default::default()
        };

        let provider = GoogleProvider::new(config).unwrap();

        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello"),
        ];

        let request = provider.build_request(&messages, None).unwrap();

        assert!(request.system_instruction.is_some());
        assert_eq!(request.contents.len(), 1); // Only user message, system is separate
    }

    #[test]
    fn test_api_url() {
        let config = ProviderConfig {
            provider: ProviderType::Google,
            api_key: Some("test-key".to_string()),
            model: "gemini-1.5-pro".to_string(),
            ..Default::default()
        };

        let provider = GoogleProvider::new(config).unwrap();

        let url = provider.api_url(false);
        assert!(url.contains("gemini-1.5-pro"));
        assert!(url.contains("generateContent"));
        assert!(url.contains("key=test-key"));

        let stream_url = provider.api_url(true);
        assert!(stream_url.contains("streamGenerateContent"));
        assert!(stream_url.contains("alt=sse"));
    }

    #[test]
    fn test_to_gemini_content_user() {
        let config = ProviderConfig {
            provider: ProviderType::Google,
            api_key: Some("test-key".to_string()),
            model: "gemini-1.5-pro".to_string(),
            ..Default::default()
        };

        let provider = GoogleProvider::new(config).unwrap();
        let msg = ChatMessage::user("Hello, world!");
        let content = provider.to_gemini_content(&msg).unwrap();

        assert_eq!(content.role, Some("user".to_string()));
        assert_eq!(content.parts.len(), 1);
    }

    #[test]
    fn test_parse_response() {
        let config = ProviderConfig {
            provider: ProviderType::Google,
            api_key: Some("test-key".to_string()),
            model: "gemini-1.5-pro".to_string(),
            ..Default::default()
        };

        let provider = GoogleProvider::new(config).unwrap();

        let response = GeminiResponse {
            candidates: vec![GeminiCandidate {
                content: GeminiContent {
                    role: Some("model".to_string()),
                    parts: vec![GeminiPart::Text {
                        text: "Hello!".to_string(),
                    }],
                },
                finish_reason: Some("STOP".to_string()),
            }],
            usage_metadata: Some(GeminiUsageMetadata {
                prompt_token_count: Some(10),
                candidates_token_count: Some(5),
            }),
        };

        let parsed = provider.parse_response(response).unwrap();

        assert_eq!(parsed.content, "Hello!");
        assert_eq!(parsed.stop_reason, StopReason::EndTurn);
        assert!(parsed.tool_calls.is_empty());
    }
}
