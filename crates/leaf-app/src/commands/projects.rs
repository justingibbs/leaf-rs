//! Project-related Tauri commands

use std::path::PathBuf;

use leaf_core::Project;
use tauri::State;
use tracing::{debug, info};

use crate::state::AppState;

/// Recent project info for the UI
#[derive(serde::Serialize)]
pub struct RecentProjectInfo {
    pub name: String,
    pub path: String,
    pub last_opened: String,
}

/// Create a new project at the given path
#[tauri::command]
pub async fn create_project(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    debug!("Creating project at: {}", path);
    let path = PathBuf::from(path);

    // Ensure the directory exists
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }

    // Open/create the project
    let project = state.open_project(path).map_err(|e| e.to_string())?;
    info!("Created project: {}", project.name);

    Ok(project)
}

/// Open an existing project
#[tauri::command]
pub async fn open_project(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    debug!("Opening project at: {}", path);
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(format!("Project path does not exist: {}", path.display()));
    }

    let project = state.open_project(path).map_err(|e| e.to_string())?;
    info!("Opened project: {}", project.name);

    Ok(project)
}

/// Get the currently open project
#[tauri::command]
pub async fn get_current_project(state: State<'_, AppState>) -> Result<Option<Project>, String> {
    state.get_current_project().map_err(|e| e.to_string())
}

/// Close the current project
#[tauri::command]
pub async fn close_project(state: State<'_, AppState>) -> Result<(), String> {
    state.close_project().map_err(|e| e.to_string())?;
    info!("Project closed");
    Ok(())
}

/// List recent projects
#[tauri::command]
pub async fn list_recent_projects(
    state: State<'_, AppState>,
) -> Result<Vec<RecentProjectInfo>, String> {
    let config = state
        .config
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let recent: Vec<RecentProjectInfo> = config
        .recent_projects
        .iter()
        .map(|p| RecentProjectInfo {
            name: p.name.clone(),
            path: p.path.clone(),
            last_opened: p.last_opened.clone(),
        })
        .collect();

    Ok(recent)
}
