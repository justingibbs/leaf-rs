//! Execution-related Tauri commands

use leaf_core::StackExecution;
use leaf_db::{StackExecutionQueries};
use tauri::State;
use tracing::debug;
use uuid::Uuid;

use crate::state::AppState;

/// List stack executions for the current project
#[tauri::command]
pub async fn list_executions(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<StackExecution>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    debug!("list_executions: project_id={}, limit={:?}", project.id, limit);

    let executions = db
        .list_stack_executions_for_project(project.id, limit)
        .map_err(|e| e.to_string())?;

    Ok(executions)
}

/// Get a single stack execution by ID
#[tauri::command]
pub async fn get_execution(
    state: State<'_, AppState>,
    execution_id: String,
) -> Result<Option<StackExecution>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let exec_uuid = Uuid::parse_str(&execution_id).map_err(|e| e.to_string())?;

    debug!("get_execution: execution_id={}", execution_id);

    let execution = db
        .get_stack_execution(exec_uuid)
        .map_err(|e| e.to_string())?;

    Ok(execution)
}

/// List stack executions for a specific event
#[tauri::command]
pub async fn list_executions_for_event(
    state: State<'_, AppState>,
    event_id: String,
) -> Result<Vec<StackExecution>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let event_uuid = Uuid::parse_str(&event_id).map_err(|e| e.to_string())?;

    debug!("list_executions_for_event: event_id={}", event_id);

    let executions = db
        .list_stack_executions_for_event(event_uuid)
        .map_err(|e| e.to_string())?;

    Ok(executions)
}
