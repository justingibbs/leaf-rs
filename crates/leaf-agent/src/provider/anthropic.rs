//! Anthropic Claude provider implementation

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

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider
pub struct AnthropicProvider {
    client: Client,
    config: ProviderConfig,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: ProviderConfig) -> AgentResult<Self> {
        let api_key = config.api_key.as_ref().ok_or_else(|| {
            AgentError::ProviderNotConfigured("Anthropic API key required".to_string())
        })?;

        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "x-api-key",
                    api_key.parse().map_err(|_| {
                        AgentError::ProviderNotConfigured("Invalid API key format".to_string())
                    })?,
                );
                headers.insert(
                    "anthropic-version",
                    ANTHROPIC_VERSION.parse().unwrap(),
                );
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                headers
            })
            .build()?;

        Ok(Self { client, config })
    }

    fn base_url(&self) -> &str {
        self.config
            .base_url
            .as_deref()
            .unwrap_or(ANTHROPIC_API_URL)
    }

    fn build_request(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
        stream: bool,
    ) -> AgentResult<AnthropicRequest> {
        // Extract system message if present
        let (system_message, chat_messages): (Option<&ChatMessage>, Vec<&ChatMessage>) = {
            let system = messages.iter().find(|m| m.role == ChatRole::System);
            let others = messages.iter().filter(|m| m.role != ChatRole::System).collect();
            (system, others)
        };

        // Convert messages to Anthropic format
        let anthropic_messages: Vec<AnthropicMessage> = chat_messages
            .into_iter()
            .map(|m| self.to_anthropic_message(m))
            .collect::<AgentResult<Vec<_>>>()?;

        // Convert tools to Anthropic format
        let anthropic_tools = tools.map(|t| {
            t.iter()
                .map(|tool| AnthropicTool {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                })
                .collect()
        });

        Ok(AnthropicRequest {
            model: self.config.model.clone(),
            messages: anthropic_messages,
            max_tokens: self.config.max_tokens,
            temperature: Some(self.config.temperature),
            system: system_message.map(|m| m.content.clone()),
            tools: anthropic_tools,
            stream: Some(stream),
        })
    }

    fn to_anthropic_message(&self, msg: &ChatMessage) -> AgentResult<AnthropicMessage> {
        let role = match msg.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => "user", // System handled separately
        };

        let mut content: Vec<AnthropicContent> = Vec::new();

        // Add tool result if present
        if let Some(ref tool_result) = msg.tool_result {
            content.push(AnthropicContent::ToolResult {
                tool_use_id: tool_result.tool_use_id.clone(),
                content: serde_json::to_string(&tool_result.content)?,
                is_error: Some(tool_result.is_error),
            });
        }

        // Add text content if present
        if !msg.content.is_empty() {
            content.push(AnthropicContent::Text {
                text: msg.content.clone(),
            });
        }

        // Add tool calls if present (for assistant messages)
        for tool_call in &msg.tool_calls {
            content.push(AnthropicContent::ToolUse {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                input: tool_call.arguments.clone(),
            });
        }

        // Ensure content is not empty
        if content.is_empty() {
            content.push(AnthropicContent::Text {
                text: String::new(),
            });
        }

        Ok(AnthropicMessage {
            role: role.to_string(),
            content,
        })
    }

    fn parse_response(&self, response: AnthropicResponse) -> ChatResponse {
        let mut text_content = String::new();
        let mut tool_calls = Vec::new();

        for block in response.content {
            match block {
                AnthropicContent::Text { text } => {
                    text_content.push_str(&text);
                }
                AnthropicContent::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCallRequest {
                        id,
                        name,
                        arguments: input,
                    });
                }
                _ => {}
            }
        }

        let stop_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("tool_use") => StopReason::ToolUse,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("stop_sequence") => StopReason::StopSequence,
            _ => StopReason::Unknown,
        };

        ChatResponse {
            content: text_content,
            tool_calls,
            stop_reason,
            usage: response.usage.map(|u| Usage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
            }),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatResponse> {
        let request = self.build_request(messages, tools, false)?;

        debug!("Sending request to Anthropic API: model={}", request.model);
        trace!("Request body: {:?}", serde_json::to_string(&request));

        let response = self
            .client
            .post(self.base_url())
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(AgentError::ProviderError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;
        debug!(
            "Received response: stop_reason={:?}",
            anthropic_response.stop_reason
        );

        Ok(self.parse_response(anthropic_response))
    }

    async fn stream_chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatStream> {
        let request = self.build_request(messages, tools, true)?;

        debug!(
            "Starting streaming request to Anthropic API: model={}",
            request.model
        );

        let response = self
            .client
            .post(self.base_url())
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(AgentError::ProviderError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        // Create SSE stream
        let byte_stream = response.bytes_stream();
        let stream = parse_sse_stream(byte_stream);

        Ok(Box::pin(stream))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Parse SSE stream from Anthropic API
fn parse_sse_stream<S>(byte_stream: S) -> impl Stream<Item = AgentResult<ChatChunk>>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + Unpin + 'static,
{
    // Buffer for incomplete SSE lines
    let buffer = String::new();
    // Track current tool use state for building partial JSON
    let current_tool_index: Option<u32> = None;
    let current_tool_id: Option<String> = None;
    let current_tool_name: Option<String> = None;

    futures::stream::unfold(
        (byte_stream, buffer, current_tool_index, current_tool_id, current_tool_name),
        |(mut stream, mut buffer, mut tool_index, mut tool_id, mut tool_name)| async move {
            use futures::StreamExt;

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
                            return None;
                        }

                        match serde_json::from_str::<AnthropicStreamEvent>(data) {
                            Ok(event) => {
                                if let Some(chunk) = process_stream_event(
                                    event,
                                    &mut tool_index,
                                    &mut tool_id,
                                    &mut tool_name,
                                ) {
                                    return Some((
                                        Ok(chunk),
                                        (stream, buffer, tool_index, tool_id, tool_name),
                                    ));
                                }
                            }
                            Err(e) => {
                                trace!("Failed to parse SSE event: {} - {}", e, data);
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
                            (stream, buffer, tool_index, tool_id, tool_name),
                        ));
                    }
                    None => {
                        // Stream ended
                        return None;
                    }
                }
            }
        },
    )
}

fn process_stream_event(
    event: AnthropicStreamEvent,
    tool_index: &mut Option<u32>,
    tool_id: &mut Option<String>,
    tool_name: &mut Option<String>,
) -> Option<ChatChunk> {
    match event {
        AnthropicStreamEvent::ContentBlockStart { index, content_block } => {
            match content_block {
                AnthropicContent::Text { .. } => {
                    Some(ChatChunk::ContentBlockStart {
                        index,
                        content_type: ContentBlockType::Text,
                    })
                }
                AnthropicContent::ToolUse { id, name, .. } => {
                    *tool_index = Some(index);
                    *tool_id = Some(id.clone());
                    *tool_name = Some(name.clone());
                    Some(ChatChunk::ContentBlockStart {
                        index,
                        content_type: ContentBlockType::ToolUse { id, name },
                    })
                }
                _ => None,
            }
        }
        AnthropicStreamEvent::ContentBlockDelta { index, delta } => {
            match delta {
                AnthropicDelta::TextDelta { text } => {
                    Some(ChatChunk::TextDelta { index, text })
                }
                AnthropicDelta::InputJsonDelta { partial_json } => {
                    Some(ChatChunk::ToolUseDelta {
                        index,
                        partial_json,
                    })
                }
            }
        }
        AnthropicStreamEvent::ContentBlockStop { index } => {
            if *tool_index == Some(index) {
                *tool_index = None;
                *tool_id = None;
                *tool_name = None;
            }
            Some(ChatChunk::ContentBlockStop { index })
        }
        AnthropicStreamEvent::MessageStop => {
            Some(ChatChunk::MessageStop {
                stop_reason: StopReason::EndTurn,
            })
        }
        AnthropicStreamEvent::MessageDelta { delta, usage } => {
            let stop_reason = delta
                .and_then(|d| d.stop_reason)
                .map(|r| match r.as_str() {
                    "end_turn" => StopReason::EndTurn,
                    "tool_use" => StopReason::ToolUse,
                    "max_tokens" => StopReason::MaxTokens,
                    _ => StopReason::Unknown,
                })
                .unwrap_or(StopReason::EndTurn);

            // Return usage if available, otherwise message stop
            if let Some(u) = usage {
                Some(ChatChunk::Usage(Usage {
                    input_tokens: u.input_tokens.unwrap_or(0),
                    output_tokens: u.output_tokens,
                }))
            } else {
                Some(ChatChunk::MessageStop { stop_reason })
            }
        }
        AnthropicStreamEvent::Error { error } => {
            Some(ChatChunk::Error {
                message: error.message,
            })
        }
        _ => None,
    }
}

// Anthropic API types

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContent {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// Streaming types

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicStreamEvent {
    MessageStart {
        #[allow(dead_code)]
        message: Option<serde_json::Value>,
    },
    ContentBlockStart {
        index: u32,
        content_block: AnthropicContent,
    },
    ContentBlockDelta {
        index: u32,
        delta: AnthropicDelta,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: Option<AnthropicMessageDelta>,
        usage: Option<AnthropicStreamUsage>,
    },
    MessageStop,
    Ping,
    Error {
        error: AnthropicError,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageDelta {
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamUsage {
    input_tokens: Option<u32>,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request() {
        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            api_key: Some("test-key".to_string()),
            model: "claude-sonnet-4-20250514".to_string(),
            ..Default::default()
        };

        let provider = AnthropicProvider::new(config).unwrap();

        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello"),
        ];

        let request = provider.build_request(&messages, None, false).unwrap();

        assert_eq!(request.model, "claude-sonnet-4-20250514");
        assert_eq!(request.system, Some("You are a helpful assistant.".to_string()));
        assert_eq!(request.messages.len(), 1); // Only user message, system is separate
    }

    #[test]
    fn test_parse_response() {
        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            api_key: Some("test-key".to_string()),
            model: "claude-sonnet-4-20250514".to_string(),
            ..Default::default()
        };

        let provider = AnthropicProvider::new(config).unwrap();

        let response = AnthropicResponse {
            content: vec![AnthropicContent::Text {
                text: "Hello!".to_string(),
            }],
            stop_reason: Some("end_turn".to_string()),
            usage: Some(AnthropicUsage {
                input_tokens: 10,
                output_tokens: 5,
            }),
        };

        let parsed = provider.parse_response(response);

        assert_eq!(parsed.content, "Hello!");
        assert_eq!(parsed.stop_reason, StopReason::EndTurn);
        assert!(parsed.tool_calls.is_empty());
    }
}
