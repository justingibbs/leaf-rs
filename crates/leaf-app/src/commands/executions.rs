//! Execution-related Tauri commands (stubbed for Phase A)
//!
//! These commands are placeholders that maintain the Tauri IPC interface.
//! They will be rewritten in Phase C with StackExecution/CardExecution logic.

use tauri::State;
use tracing::debug;

use crate::state::AppState;

/// List executions (stubbed - returns empty)
/// TODO: Phase C - rewrite to list stack executions
#[tauri::command]
pub async fn list_executions(
    state: State<'_, AppState>,
    card_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    debug!("list_executions: stubbed for Phase A, card_id: {:?}, limit: {:?}", card_id, limit);
    let _ = state;
    Ok(Vec::new())
}

/// Get a single execution by ID (stubbed - returns None)
/// TODO: Phase C - rewrite to get stack execution
#[tauri::command]
pub async fn get_execution(
    state: State<'_, AppState>,
    execution_id: String,
) -> Result<Option<serde_json::Value>, String> {
    debug!("get_execution: stubbed for Phase A, execution_id={}", execution_id);
    let _ = state;
    Ok(None)
}

/// List executions for a specific event (stubbed - returns empty)
/// TODO: Phase C - rewrite to list stack executions for event
#[tauri::command]
pub async fn list_executions_for_event(
    state: State<'_, AppState>,
    event_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    debug!("list_executions_for_event: stubbed for Phase A, event_id={}", event_id);
    let _ = state;
    Ok(Vec::new())
}
