//! Artifact-related Tauri commands

use leaf_core::{Artifact, ArtifactStatus, ArtifactType, LeafEvent};
use leaf_db::ArtifactQueries;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::state::AppState;

/// List artifacts for the current project
#[tauri::command]
pub async fn list_artifacts(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<Artifact>, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    let artifacts = if let Some(status_str) = status {
        let artifact_status: ArtifactStatus =
            serde_json::from_str(&format!("\"{}\"", status_str))
                .map_err(|e| format!("Invalid status: {}", e))?;
        db.list_artifacts_by_status(project.id, &artifact_status)
            .map_err(|e| e.to_string())?
    } else {
        db.list_artifacts_for_project(project.id)
            .map_err(|e| e.to_string())?
    };

    Ok(artifacts)
}

/// Get a single artifact by ID
#[tauri::command]
pub async fn get_artifact(
    state: State<'_, AppState>,
    artifact_id: String,
) -> Result<Option<Artifact>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let uuid = Uuid::parse_str(&artifact_id).map_err(|e| e.to_string())?;
    let artifact = db.get_artifact(uuid).map_err(|e| e.to_string())?;
    Ok(artifact)
}

/// Scan the project directory and register existing files as artifacts
#[tauri::command]
pub async fn scan_artifacts(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<Artifact>, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let project_path = state.get_project_path().map_err(|e| e.to_string())?;

    let mut registered = Vec::new();

    for entry in WalkDir::new(&project_path)
        .into_iter()
        .filter_entry(|e| {
            // Skip .leaf directory and hidden dirs
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') || e.depth() == 0
        })
        .filter_map(|e| e.ok())
    {
        // Skip the root directory itself
        if entry.depth() == 0 {
            continue;
        }

        let path = entry.path();
        let relative_path = path
            .strip_prefix(&project_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Check if already tracked
        if db
            .get_artifact_by_path(project.id, &relative_path)
            .map_err(|e| e.to_string())?
            .is_some()
        {
            continue;
        }

        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let artifact_type = if path.is_dir() {
            ArtifactType::Directory
        } else {
            ArtifactType::File
        };

        let mut artifact = Artifact::new(
            project.id,
            &relative_path,
            &filename,
            artifact_type,
            "user",
        );

        if path.is_file() {
            artifact.mime_type = mime_guess::from_path(path)
                .first()
                .map(|m| m.to_string());
            artifact.size_bytes = std::fs::metadata(path).ok().map(|m| m.len() as i64);
        }

        db.create_artifact(&artifact).map_err(|e| e.to_string())?;

        if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::ArtifactCreated(artifact.clone()))
        {
            warn!("Failed to emit ArtifactCreated event: {}", e);
        }

        registered.push(artifact);
    }

    info!(
        "Scanned project directory, registered {} new artifacts",
        registered.len()
    );

    Ok(registered)
}
