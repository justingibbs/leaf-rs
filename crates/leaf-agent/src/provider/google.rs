//! Google Gemini provider implementation (stub for future)

use async_trait::async_trait;

use super::{
    ChatMessage, ChatResponse, ChatStream, LlmProvider, ProviderConfig, ProviderType,
    ToolDefinition,
};
use crate::error::{AgentError, AgentResult};

/// Google Gemini provider (stub implementation)
pub struct GoogleProvider {
    config: ProviderConfig,
}

impl GoogleProvider {
    /// Create a new Google provider
    pub fn new(config: ProviderConfig) -> AgentResult<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn chat(
        &self,
        _messages: &[ChatMessage],
        _tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatResponse> {
        Err(AgentError::ProviderNotConfigured(
            "Google provider not yet implemented".to_string(),
        ))
    }

    async fn stream_chat(
        &self,
        _messages: &[ChatMessage],
        _tools: Option<&[ToolDefinition]>,
    ) -> AgentResult<ChatStream> {
        Err(AgentError::ProviderNotConfigured(
            "Google provider not yet implemented".to_string(),
        ))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Google
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}
