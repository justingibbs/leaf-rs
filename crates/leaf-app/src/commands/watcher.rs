//! Watcher-related Tauri commands

use leaf_core::WatchPath;
use tauri::{AppHandle, State};
use tracing::{debug, info};

use crate::state::AppState;

/// Start the file watcher for the current project
#[tauri::command]
pub async fn start_watcher(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    debug!("Starting file watcher");

    let paths = state
        .start_watcher(app)
        .map_err(|e| e.to_string())?;

    let path_strings: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    info!("File watcher started with {} paths", path_strings.len());
    Ok(path_strings)
}

/// Stop the file watcher for the current project
#[tauri::command]
pub async fn stop_watcher(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Stopping file watcher");
    state.stop_watcher(app).map_err(|e| e.to_string())?;
    info!("File watcher stopped");
    Ok(())
}

/// Check if the watcher is currently running
#[tauri::command]
pub async fn is_watcher_running(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.is_watcher_running().map_err(|e| e.to_string())
}

/// Get the list of configured watch paths
#[tauri::command]
pub async fn get_watch_paths(
    state: State<'_, AppState>,
) -> Result<Vec<WatchPath>, String> {
    state.get_watch_paths().map_err(|e| e.to_string())
}

/// Add a new watch path
#[tauri::command]
pub async fn add_watch_path(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    patterns: Vec<String>,
) -> Result<(), String> {
    debug!("Adding watch path: {}", path);
    state
        .add_watch_path(app, path.clone(), patterns)
        .map_err(|e| e.to_string())?;
    info!("Added watch path: {}", path);
    Ok(())
}

/// Remove a watch path
#[tauri::command]
pub async fn remove_watch_path(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    debug!("Removing watch path: {}", path);
    state
        .remove_watch_path(app, path.clone())
        .map_err(|e| e.to_string())?;
    info!("Removed watch path: {}", path);
    Ok(())
}
