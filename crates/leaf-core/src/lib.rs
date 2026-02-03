//! LEAF Core - Core types, traits, and utilities for LEAF
//!
//! This crate provides the foundational types used throughout the LEAF application:
//! - `Project`, `Card`, `Event`, `Execution` - Core domain types
//! - `ChatSession`, `Message` - AI chat types
//! - Error types and configuration structs

pub mod config;
pub mod error;
pub mod events;
pub mod types;

pub use config::{AppConfig, ExecutionSettings, LlmSettings, ProjectConfig, RecentProject, WatchPath};
pub use error::{LeafError, Result};
pub use events::{EventBus, EventSubscription, LeafEvent};
pub use types::*;
