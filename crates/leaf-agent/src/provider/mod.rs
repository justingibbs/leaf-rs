//! LLM Provider abstraction layer
//!
//! This module provides a trait for interacting with different LLM providers
//! (Anthropic, OpenAI, Google, Ollama) and related types.

pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::error::AgentResult;

/// Configuration for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type (anthropic, openai, google, ollama)
    pub provider: ProviderType,
    /// API key (not required for Ollama)
    pub api_key: Option<String>,
    /// Model identifier (e.g., "claude-sonnet-4-20250514", "gpt-4")
    pub model: String,
    /// Base URL override (useful for proxies or Ollama)
    pub base_url: Option<String>,
    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature for sampling (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: ProviderType::Anthropic,
            api_key: None,
            model: "claude-sonnet-4-20250514".to_string(),
            base_url: None,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Google,
    Ollama,
}

/// Role in a chat conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

/// A message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: ChatRole,
    /// Text content of the message
    pub content: String,
    /// Tool calls made in this message (for assistant messages)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCallRequest>,
    /// Tool result (for messages following tool calls)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
}

impl ChatMessage {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_result: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_result: None,
        }
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_result: None,
        }
    }

    /// Create a message with a tool result
    pub fn tool_result(tool_use_id: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            role: ChatRole::User,
            content: String::new(),
            tool_calls: Vec::new(),
            tool_result: Some(ToolResult {
                tool_use_id: tool_use_id.into(),
                content: result,
                is_error: false,
            }),
        }
    }

    /// Create a message with a tool error
    pub fn tool_error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: String::new(),
            tool_calls: Vec::new(),
            tool_result: Some(ToolResult {
                tool_use_id: tool_use_id.into(),
                content: serde_json::json!({ "error": error.into() }),
                is_error: true,
            }),
        }
    }
}

/// A request from the LLM to call a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Unique ID for this tool call
    pub id: String,
    /// Name of the tool to call
    pub name: String,
    /// Arguments for the tool call
    pub arguments: serde_json::Value,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this result is for
    pub tool_use_id: String,
    /// Result content (JSON)
    pub content: serde_json::Value,
    /// Whether this is an error result
    pub is_error: bool,
}

/// Definition of a tool the LLM can use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// JSON Schema for the tool's input parameters
    pub input_schema: serde_json::Value,
}

/// Response from a chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Text content of the response
    pub content: String,
    /// Tool calls requested by the model
    pub tool_calls: Vec<ToolCallRequest>,
    /// Stop reason
    pub stop_reason: StopReason,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Reason the model stopped generating
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Model finished naturally
    EndTurn,
    /// Model wants to use a tool
    ToolUse,
    /// Maximum tokens reached
    MaxTokens,
    /// Stop sequence encountered
    StopSequence,
    /// Unknown reason
    Unknown,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Input tokens consumed
    pub input_tokens: u32,
    /// Output tokens generated
    pub output_tokens: u32,
}

/// A chunk of a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatChunk {
    /// Start of a content block
    ContentBlockStart {
        index: u32,
        content_type: ContentBlockType,
    },
    /// Text delta
    TextDelta {
        index: u32,
        text: String,
    },
    /// Tool use input delta (partial JSON)
    ToolUseDelta {
        index: u32,
        partial_json: String,
    },
    /// End of a content block
    ContentBlockStop {
        index: u32,
    },
    /// Message complete
    MessageStop {
        stop_reason: StopReason,
    },
    /// Usage update
    Usage(Usage),
    /// Error
    Error {
        message: String,
    },
}

/// Type of content block in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentBlockType {
    Text,
    ToolUse { id: String, name: String },
}

/// Stream of chat response chunks
pub type ChatStream = Pin<Box<dyn Stream<Item = AgentResult<ChatChunk>> + Send>>;

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a chat request and get a complete response
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatResponse>;

    /// Send a chat request and get a streaming response
    async fn stream_chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatStream>;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Get the model being used
    fn model(&self) -> &str;
}

/// Create an LLM provider from configuration
pub fn create_provider(config: ProviderConfig) -> AgentResult<Box<dyn LlmProvider>> {
    match config.provider {
        ProviderType::Anthropic => {
            let provider = anthropic::AnthropicProvider::new(config)?;
            Ok(Box::new(provider))
        }
        ProviderType::OpenAI => {
            let provider = openai::OpenAIProvider::new(config)?;
            Ok(Box::new(provider))
        }
        ProviderType::Google => {
            let provider = google::GoogleProvider::new(config)?;
            Ok(Box::new(provider))
        }
        ProviderType::Ollama => {
            let provider = ollama::OllamaProvider::new(config)?;
            Ok(Box::new(provider))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_user() {
        let msg = ChatMessage::user("Hello");
        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_chat_message_tool_result() {
        let msg = ChatMessage::tool_result("call_123", serde_json::json!({"result": "ok"}));
        assert!(msg.tool_result.is_some());
        let result = msg.tool_result.unwrap();
        assert_eq!(result.tool_use_id, "call_123");
        assert!(!result.is_error);
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.provider, ProviderType::Anthropic);
        assert_eq!(config.max_tokens, 4096);
    }
}
