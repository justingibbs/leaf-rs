//! Executor error types

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during execution
#[derive(Error, Debug)]
pub enum ExecutorError {
    /// Deno runtime not found on the system
    #[error("Deno not found. Install Deno: https://deno.land/manual/getting_started/installation")]
    DenoNotFound,

    /// Program file not found
    #[error("Program not found: {0}")]
    ProgramNotFound(PathBuf),

    /// Execution timed out
    #[error("Execution timed out after {0} seconds")]
    Timeout(u32),

    /// Execution failed with an error
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Process spawn or I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid project path
    #[error("Invalid project path: {0}")]
    InvalidProjectPath(String),
}

/// Result type for executor operations
pub type Result<T> = std::result::Result<T, ExecutorError>;
