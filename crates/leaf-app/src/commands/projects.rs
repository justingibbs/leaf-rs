//! Project-related Tauri commands

use std::path::PathBuf;

use leaf_core::Project;
use tauri::{AppHandle, State};
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
///
/// This will create the project directory if it doesn't exist,
/// initialize the .leaf folder, and automatically start the file watcher.
#[tauri::command]
pub async fn create_project(
    app: AppHandle,
    path: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    debug!("Creating project at: {}", path);
    let path = PathBuf::from(path);

    // Ensure the directory exists
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }

    // Open/create the project (watcher auto-starts per spec)
    let project = state.open_project(path, app).map_err(|e| e.to_string())?;
    info!("Created project: {}", project.name);

    Ok(project)
}

/// Open an existing project
///
/// This will open the project and automatically start the file watcher
/// per the spec's synchronization: `Project.open() -> Watcher.start()`.
#[tauri::command]
pub async fn open_project(
    app: AppHandle,
    path: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    debug!("Opening project at: {}", path);
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(format!("Project path does not exist: {}", path.display()));
    }

    // Open project (watcher auto-starts per spec)
    let project = state.open_project(path, app).map_err(|e| e.to_string())?;
    info!("Opened project: {}", project.name);

    Ok(project)
}

/// Get the currently open project
#[tauri::command]
pub async fn get_current_project(state: State<'_, AppState>) -> Result<Option<Project>, String> {
    state.get_current_project().map_err(|e| e.to_string())
}

/// Close the current project
///
/// This will stop the file watcher and close the project.
#[tauri::command]
pub async fn close_project(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Stop the watcher first (per spec: Project.close() -> Watcher.stop())
    let _ = state.stop_watcher(app);

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
