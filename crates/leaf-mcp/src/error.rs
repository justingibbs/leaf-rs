//! Error types for the MCP client

use thiserror::Error;

/// MCP client error type
#[derive(Debug, Error)]
pub enum McpError {
    /// Transport error (connection, I/O)
    #[error("Transport error: {0}")]
    Transport(String),

    /// Protocol error (invalid messages, unexpected responses)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Server not found in registry
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    /// Server not connected
    #[error("Server not connected: {0}")]
    ServerNotConnected(String),

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    /// Process error (spawn, termination)
    #[error("Process error: {0}")]
    ProcessError(String),

    /// Timeout error
    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    /// JSON-RPC error from server
    #[error("JSON-RPC error {code}: {message}")]
    JsonRpcError { code: i32, message: String },

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Server initialization failed
    #[error("Server initialization failed: {0}")]
    InitializationFailed(String),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;
