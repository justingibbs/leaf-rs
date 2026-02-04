//! Execution-related Tauri commands

use leaf_core::Execution;
use leaf_db::ExecutionQueries;
use tauri::State;
use tracing::debug;
use uuid::Uuid;

use crate::state::AppState;

/// List executions, optionally filtered by card
#[tauri::command]
pub async fn list_executions(
    state: State<'_, AppState>,
    card_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Execution>, String> {
    debug!("Listing executions, card_id: {:?}, limit: {:?}", card_id, limit);

    let db = state.get_db().map_err(|e| e.to_string())?;

    let executions = if let Some(card_id_str) = card_id {
        let id = Uuid::parse_str(&card_id_str).map_err(|e| format!("Invalid card ID: {}", e))?;
        db.list_executions_for_card(id, limit)
            .map_err(|e| format!("Failed to list executions: {}", e))?
    } else {
        // List all running executions if no card specified
        db.list_running_executions()
            .map_err(|e| format!("Failed to list executions: {}", e))?
    };

    debug!("Found {} executions", executions.len());
    Ok(executions)
}

/// Get a single execution by ID
#[tauri::command]
pub async fn get_execution(
    state: State<'_, AppState>,
    execution_id: String,
) -> Result<Option<Execution>, String> {
    debug!("Getting execution: {}", execution_id);

    let id = Uuid::parse_str(&execution_id).map_err(|e| format!("Invalid execution ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    let execution = db
        .get_execution(id)
        .map_err(|e| format!("Failed to get execution: {}", e))?;

    Ok(execution)
}

/// List executions for a specific event
#[tauri::command]
pub async fn list_executions_for_event(
    state: State<'_, AppState>,
    event_id: String,
) -> Result<Vec<Execution>, String> {
    debug!("Listing executions for event: {}", event_id);

    let id = Uuid::parse_str(&event_id).map_err(|e| format!("Invalid event ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    let executions = db
        .list_executions_for_event(id)
        .map_err(|e| format!("Failed to list executions: {}", e))?;

    debug!("Found {} executions", executions.len());
    Ok(executions)
}
