//! Error types for the watcher crate

use std::path::PathBuf;
use thiserror::Error;

/// Result type for watcher operations
pub type WatcherResult<T> = Result<T, WatcherError>;

/// Errors that can occur in the file watcher
#[derive(Debug, Error)]
pub enum WatcherError {
    /// Failed to create the file watcher
    #[error("Failed to create watcher: {0}")]
    WatcherCreation(String),

    /// Failed to watch a path
    #[error("Failed to watch path {path}: {reason}")]
    WatchPath { path: PathBuf, reason: String },

    /// Failed to unwatch a path
    #[error("Failed to unwatch path {path}: {reason}")]
    UnwatchPath { path: PathBuf, reason: String },

    /// Path not found
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    /// Path is not a directory
    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    /// Invalid glob pattern
    #[error("Invalid glob pattern '{pattern}': {reason}")]
    InvalidPattern { pattern: String, reason: String },

    /// Watcher is not running
    #[error("Watcher is not running")]
    NotRunning,

    /// Watcher is already running
    #[error("Watcher is already running")]
    AlreadyRunning,

    /// Internal notify error
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    /// Channel send error
    #[error("Failed to send event: channel closed")]
    ChannelClosed,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
