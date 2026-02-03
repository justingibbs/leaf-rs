//! Error types for LEAF

use thiserror::Error;

/// Main error type for LEAF operations
#[derive(Error, Debug)]
pub enum LeafError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Card not found: {0}")]
    CardNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Execution not found: {0}")]
    ExecutionNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Watcher error: {0}")]
    Watcher(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Execution timeout")]
    ExecutionTimeout,

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

#[cfg(feature = "database")]
impl From<rusqlite::Error> for LeafError {
    fn from(err: rusqlite::Error) -> Self {
        LeafError::Database(err.to_string())
    }
}

/// Result type alias for LEAF operations
pub type Result<T> = std::result::Result<T, LeafError>;

impl LeafError {
    /// Create a database error
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    /// Create a project not found error
    pub fn project_not_found(id: impl Into<String>) -> Self {
        Self::ProjectNotFound(id.into())
    }

    /// Create a card not found error
    pub fn card_not_found(id: impl Into<String>) -> Self {
        Self::CardNotFound(id.into())
    }

    /// Create a session not found error
    pub fn session_not_found(id: impl Into<String>) -> Self {
        Self::SessionNotFound(id.into())
    }

    /// Create an invalid config error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a filesystem error
    pub fn filesystem(msg: impl Into<String>) -> Self {
        Self::FileSystem(msg.into())
    }

    /// Create an execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::Execution(msg.into())
    }
}

/// Convert LeafError to a string for Tauri commands
impl From<LeafError> for String {
    fn from(err: LeafError) -> Self {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LeafError::project_not_found("123");
        assert_eq!(err.to_string(), "Project not found: 123");
    }

    #[test]
    fn test_error_into_string() {
        let err = LeafError::database("connection failed");
        let s: String = err.into();
        assert!(s.contains("Database error"));
    }
}
