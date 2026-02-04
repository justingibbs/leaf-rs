//! Settings-related Tauri commands

use crate::state::AppState;
use leaf_core::{AppConfig, ExecutionSettings, LlmSettings, ProjectConfig, WatchPath};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

/// Partial update for app configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigUpdate {
    pub default_llm_provider: Option<String>,
    pub theme: Option<String>,
    pub log_level: Option<String>,
}

/// Partial update for project configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfigUpdate {
    pub name: Option<String>,
    pub llm_settings: Option<LlmSettingsUpdate>,
    pub execution_settings: Option<ExecutionSettingsUpdate>,
}

/// Partial update for LLM settings
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmSettingsUpdate {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Partial update for execution settings
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSettingsUpdate {
    pub timeout_secs: Option<u32>,
    pub max_retries: Option<u32>,
    pub allow_network: Option<bool>,
    pub deno_permissions: Option<Vec<String>>,
}

/// Frontend-friendly app config response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigResponse {
    pub data_dir: String,
    pub log_dir: String,
    pub log_level: String,
    pub default_llm_provider: String,
    pub theme: String,
}

impl From<&AppConfig> for AppConfigResponse {
    fn from(config: &AppConfig) -> Self {
        Self {
            data_dir: config.data_dir.to_string_lossy().to_string(),
            log_dir: config.log_dir.to_string_lossy().to_string(),
            log_level: config.log_level.clone(),
            default_llm_provider: config.default_llm_provider.clone(),
            theme: config.theme.clone(),
        }
    }
}

/// Frontend-friendly project config response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfigResponse {
    pub name: String,
    pub watch_paths: Vec<WatchPath>,
    pub llm_settings: LlmSettings,
    pub execution_settings: ExecutionSettings,
}

impl From<&ProjectConfig> for ProjectConfigResponse {
    fn from(config: &ProjectConfig) -> Self {
        Self {
            name: config.name.clone(),
            watch_paths: config.watch_paths.clone(),
            llm_settings: config.llm_settings.clone(),
            execution_settings: config.execution_settings.clone(),
        }
    }
}

/// Get the application configuration
#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> Result<AppConfigResponse, String> {
    let config = state
        .config
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;

    Ok(AppConfigResponse::from(&*config))
}

/// Update the application configuration
#[tauri::command]
pub async fn update_app_config(
    state: State<'_, AppState>,
    updates: AppConfigUpdate,
) -> Result<AppConfigResponse, String> {
    let mut config = state
        .config
        .write()
        .map_err(|e| format!("Failed to write config: {}", e))?;

    // Apply updates
    if let Some(provider) = updates.default_llm_provider {
        config.default_llm_provider = provider;
    }
    if let Some(theme) = updates.theme {
        config.theme = theme;
    }
    if let Some(log_level) = updates.log_level {
        config.log_level = log_level;
    }

    // Save to disk
    let config_path = config.data_dir.join("config.json");
    let content =
        serde_json::to_string_pretty(&*config).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;

    info!("App config updated and saved to {}", config_path.display());

    Ok(AppConfigResponse::from(&*config))
}

/// Get the current project's configuration
#[tauri::command]
pub async fn get_project_settings(
    state: State<'_, AppState>,
) -> Result<ProjectConfigResponse, String> {
    let current = state
        .current_project
        .read()
        .map_err(|e| format!("Failed to read state: {}", e))?;

    let open_project = current
        .as_ref()
        .ok_or_else(|| "No project open".to_string())?;

    Ok(ProjectConfigResponse::from(&open_project.config))
}

/// Update the current project's configuration
#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, AppState>,
    updates: ProjectConfigUpdate,
) -> Result<ProjectConfigResponse, String> {
    let mut current = state
        .current_project
        .write()
        .map_err(|e| format!("Failed to write state: {}", e))?;

    let open_project = current
        .as_mut()
        .ok_or_else(|| "No project open".to_string())?;

    // Apply updates
    if let Some(name) = updates.name {
        open_project.config.name = name;
    }

    if let Some(llm) = updates.llm_settings {
        if let Some(provider) = llm.provider {
            open_project.config.llm_settings.provider = provider;
        }
        if let Some(model) = llm.model {
            open_project.config.llm_settings.model = model;
        }
        if let Some(api_key) = llm.api_key {
            // Allow clearing the API key with empty string
            if api_key.is_empty() {
                open_project.config.llm_settings.api_key = None;
            } else {
                open_project.config.llm_settings.api_key = Some(api_key);
            }
        }
        if let Some(base_url) = llm.base_url {
            if base_url.is_empty() {
                open_project.config.llm_settings.base_url = None;
            } else {
                open_project.config.llm_settings.base_url = Some(base_url);
            }
        }
        if let Some(temperature) = llm.temperature {
            open_project.config.llm_settings.temperature = temperature;
        }
        if let Some(max_tokens) = llm.max_tokens {
            open_project.config.llm_settings.max_tokens = max_tokens;
        }
    }

    if let Some(exec) = updates.execution_settings {
        if let Some(timeout) = exec.timeout_secs {
            open_project.config.execution_settings.timeout_secs = timeout;
        }
        if let Some(retries) = exec.max_retries {
            open_project.config.execution_settings.max_retries = retries;
        }
        if let Some(allow_network) = exec.allow_network {
            open_project.config.execution_settings.allow_network = allow_network;
        }
        if let Some(permissions) = exec.deno_permissions {
            open_project.config.execution_settings.deno_permissions = permissions;
        }
    }

    // Save config
    let config_path = open_project.path.join(".leaf").join("config.json");
    let content = serde_json::to_string_pretty(&open_project.config)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;

    info!(
        "Project config updated and saved to {}",
        config_path.display()
    );

    Ok(ProjectConfigResponse::from(&open_project.config))
}
