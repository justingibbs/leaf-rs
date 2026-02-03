//! LEAF Watcher - File system monitoring for LEAF
//!
//! This crate provides file watching capabilities using the `notify` crate.
//! It monitors folders for file creation/modification events and emits
//! LEAF events when files match configured patterns.
//!
//! # Features
//!
//! - Real-time file system monitoring using native OS APIs
//! - Glob pattern matching for filtering files
//! - Configurable debouncing to prevent duplicate events
//! - Async event stream via tokio channels
//!
//! # Example
//!
//! ```no_run
//! use leaf_watcher::{FileWatcher, WatchConfig};
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let mut watcher = FileWatcher::new(tx).unwrap();
//!
//!     watcher.watch(WatchConfig {
//!         path: "/path/to/watch".into(),
//!         patterns: vec!["*.pdf".to_string()],
//!         debounce_ms: 500,
//!     }).unwrap();
//!
//!     while let Some(event) = rx.recv().await {
//!         println!("Got event: {:?}", event);
//!     }
//! }
//! ```

mod debouncer;
mod error;
mod matcher;
mod watcher;

pub use debouncer::Debouncer;
pub use error::{WatcherError, WatcherResult};
pub use matcher::PatternMatcher;
pub use watcher::{FileWatcher, WatchConfig, WatchEvent, WatchEventKind};
