//! Agent-specific error types

use thiserror::Error;

/// Errors that can occur in the agent module
#[derive(Debug, Error)]
pub enum AgentError {
    /// No LLM provider has been configured
    #[error("LLM provider not configured: {0}")]
    ProviderNotConfigured(String),

    /// Error from the LLM provider
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Error during tool execution
    #[error("Tool error: {0}")]
    ToolError(String),

    /// Error during code generation
    #[error("Code generation error: {0}")]
    CodeGenerationError(String),

    /// Invalid tool call from the LLM
    #[error("Invalid tool call: {0}")]
    InvalidToolCall(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Result type for agent operations
pub type AgentResult<T> = std::result::Result<T, AgentError>;

impl From<leaf_core::LeafError> for AgentError {
    fn from(err: leaf_core::LeafError) -> Self {
        AgentError::DatabaseError(err.to_string())
    }
}
