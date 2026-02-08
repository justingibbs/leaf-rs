//! Configuration types for LEAF

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Application-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Directory for app data (databases, logs, etc.)
    pub data_dir: PathBuf,
    /// Directory for app logs
    pub log_dir: PathBuf,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Recent projects list
    #[serde(default)]
    pub recent_projects: Vec<RecentProject>,
    /// Default LLM provider
    #[serde(default = "default_llm_provider")]
    pub default_llm_provider: String,
    /// Theme (light, dark, system)
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_llm_provider() -> String {
    "anthropic".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("leaf");
        let log_dir = data_dir.join("logs");

        Self {
            data_dir,
            log_dir,
            log_level: "info".to_string(),
            recent_projects: Vec::new(),
            default_llm_provider: default_llm_provider(),
            theme: default_theme(),
        }
    }
}

/// A recently opened project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    pub last_opened: String,
}

/// Project-specific configuration (stored in .leaf/config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Stable project ID (persisted across app restarts)
    #[serde(default = "Uuid::new_v4")]
    pub project_id: Uuid,
    /// Project display name
    pub name: String,
    /// Watched folders for file events
    #[serde(default)]
    pub watch_paths: Vec<WatchPath>,
    /// LLM provider settings for this project
    #[serde(default)]
    pub llm_settings: LlmSettings,
    /// Execution settings
    #[serde(default)]
    pub execution_settings: ExecutionSettings,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_id: Uuid::new_v4(),
            name: "Untitled Project".to_string(),
            watch_paths: Vec::new(),
            llm_settings: LlmSettings::default(),
            execution_settings: ExecutionSettings::default(),
        }
    }
}

impl ProjectConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// A watched folder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchPath {
    /// Path to watch (relative to project or absolute)
    pub path: String,
    /// Glob patterns to match (e.g., "*.pdf", "**/*.txt")
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Whether this watch is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Debounce time in milliseconds
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
}

fn default_true() -> bool {
    true
}

fn default_debounce() -> u64 {
    500
}

/// LLM provider settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    /// Provider name (anthropic, openai, google, ollama)
    pub provider: String,
    /// Model to use
    pub model: String,
    /// API key (will be encrypted/stored securely)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL for API (for ollama or custom endpoints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Max tokens for response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    4096
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: None,
            base_url: None,
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

/// Execution environment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSettings {
    /// Default timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    /// Default max retries
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    /// Whether to allow network access in sandbox
    #[serde(default)]
    pub allow_network: bool,
    /// Additional Deno permissions
    #[serde(default)]
    pub deno_permissions: Vec<String>,
}

fn default_timeout() -> u32 {
    300
}

fn default_retries() -> u32 {
    3
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            max_retries: default_retries(),
            allow_network: false,
            deno_permissions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.theme, "system");
    }

    #[test]
    fn test_project_config_serialization() {
        let config = ProjectConfig::new("Test Project");
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ProjectConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test Project");
    }

    #[test]
    fn test_llm_settings_default() {
        let settings = LlmSettings::default();
        assert_eq!(settings.provider, "anthropic");
        assert!(settings.api_key.is_none());
    }
}
