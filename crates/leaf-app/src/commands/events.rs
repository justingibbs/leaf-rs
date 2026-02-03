//! Event-related Tauri commands

use leaf_core::Event;
use leaf_db::EventQueries;
use tauri::State;
use tracing::debug;
use uuid::Uuid;

use crate::state::AppState;

/// List recent events for the current project
#[tauri::command]
pub async fn list_events(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<Event>, String> {
    debug!("Listing events with limit: {:?}", limit);

    let current = state
        .current_project
        .read()
        .map_err(|e| format!("Failed to read state: {}", e))?;

    let open_project = current
        .as_ref()
        .ok_or_else(|| "No project open".to_string())?;

    let events = open_project
        .db
        .list_events(open_project.project.id, limit)
        .map_err(|e| format!("Failed to list events: {}", e))?;

    debug!("Found {} events", events.len());
    Ok(events)
}

/// Get a specific event by ID
#[tauri::command]
pub async fn get_event(
    state: State<'_, AppState>,
    event_id: String,
) -> Result<Option<Event>, String> {
    debug!("Getting event: {}", event_id);

    let id = Uuid::parse_str(&event_id)
        .map_err(|e| format!("Invalid event ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let event = db.get_event(id).map_err(|e| format!("Failed to get event: {}", e))?;

    Ok(event)
}

/// List pending events for the current project
#[tauri::command]
pub async fn list_pending_events(
    state: State<'_, AppState>,
) -> Result<Vec<Event>, String> {
    debug!("Listing pending events");

    let current = state
        .current_project
        .read()
        .map_err(|e| format!("Failed to read state: {}", e))?;

    let open_project = current
        .as_ref()
        .ok_or_else(|| "No project open".to_string())?;

    let events = open_project
        .db
        .list_pending_events(open_project.project.id)
        .map_err(|e| format!("Failed to list pending events: {}", e))?;

    debug!("Found {} pending events", events.len());
    Ok(events)
}
