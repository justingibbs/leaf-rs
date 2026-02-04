//! Chat session Tauri commands

use leaf_core::{ChatSession, LeafEvent, SessionStatus};
use leaf_db::SessionQueries;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// List all chat sessions for the current project
#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let sessions = db
        .list_sessions(project.id)
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

/// List active (non-archived) sessions for the current project
#[tauri::command]
pub async fn list_active_sessions(state: State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let sessions = db
        .list_active_sessions(project.id)
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

/// Create a new chat session
#[tauri::command]
pub async fn create_session(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    title: Option<String>,
) -> Result<ChatSession, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    let session = ChatSession::new(project.id, title.unwrap_or_else(|| "New Chat".to_string()));

    db.create_session(&session).map_err(|e| e.to_string())?;

    info!("Created session: {} ({})", session.title, session.id);

    // Emit session created event
    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::SessionCreated(session.clone())) {
        warn!("Failed to emit SessionCreated event: {}", e);
    }

    Ok(session)
}

/// Get a chat session by ID
#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<ChatSession>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let session = db.get_session(session_uuid).map_err(|e| e.to_string())?;

    Ok(session)
}

/// Update a session's title
#[tauri::command]
pub async fn update_session_title(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    title: String,
) -> Result<ChatSession, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let mut session = db
        .get_session(session_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Session not found")?;

    session.title = title;
    session.updated_at = chrono::Utc::now();

    db.update_session(&session).map_err(|e| e.to_string())?;

    info!("Updated session title: {} ({})", session.title, session.id);

    // Emit session updated event
    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::SessionUpdated(session.clone())) {
        warn!("Failed to emit SessionUpdated event: {}", e);
    }

    Ok(session)
}

/// Archive a session (hide from main list)
#[tauri::command]
pub async fn archive_session(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ChatSession, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let mut session = db
        .get_session(session_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Session not found")?;

    session.status = SessionStatus::Archived;
    session.updated_at = chrono::Utc::now();

    db.update_session(&session).map_err(|e| e.to_string())?;

    info!("Archived session: {} ({})", session.title, session.id);

    // Emit session updated event
    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::SessionUpdated(session.clone())) {
        warn!("Failed to emit SessionUpdated event: {}", e);
    }

    Ok(session)
}

/// Unarchive a session
#[tauri::command]
pub async fn unarchive_session(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ChatSession, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let mut session = db
        .get_session(session_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Session not found")?;

    session.status = SessionStatus::Active;
    session.updated_at = chrono::Utc::now();

    db.update_session(&session).map_err(|e| e.to_string())?;

    info!("Unarchived session: {} ({})", session.title, session.id);

    // Emit session updated event
    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::SessionUpdated(session.clone())) {
        warn!("Failed to emit SessionUpdated event: {}", e);
    }

    Ok(session)
}

/// Delete a session and all its messages
#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    use leaf_db::MessageQueries;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    // Delete messages first (foreign key constraint)
    db.delete_messages_for_session(session_uuid)
        .map_err(|e| e.to_string())?;

    // Delete the session
    db.delete_session(session_uuid)
        .map_err(|e| e.to_string())?;

    info!("Deleted session: {}", session_id);

    Ok(())
}
