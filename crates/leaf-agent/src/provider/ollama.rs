//! Ollama local provider implementation (stub for future)

use async_trait::async_trait;

use super::{
    ChatMessage, ChatResponse, ChatStream, LlmProvider, ProviderConfig, ProviderType,
    ToolDefinition,
};
use crate::error::{AgentError, AgentResult};

/// Ollama local provider (stub implementation)
pub struct OllamaProvider {
    config: ProviderConfig,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(config: ProviderConfig) -> AgentResult<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(
        &self,
        _messages: &[ChatMessage],
        _tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatResponse> {
        Err(AgentError::ProviderNotConfigured(
            "Ollama provider not yet implemented".to_string(),
        ))
    }

    async fn stream_chat(
        &self,
        _messages: &[ChatMessage],
        _tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatStream> {
        Err(AgentError::ProviderNotConfigured(
            "Ollama provider not yet implemented".to_string(),
        ))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}
