//! Stack-related Tauri commands

use leaf_core::{LeafEvent, Stack, TriggerConfig};
use leaf_db::StackQueries;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Input for creating a new stack
#[derive(Debug, serde::Deserialize)]
pub struct CreateStackInput {
    pub name: String,
    pub description: String,
    pub trigger: TriggerConfig,
    pub source_session_id: Option<String>,
}

/// Input for updating an existing stack
#[derive(Debug, serde::Deserialize)]
pub struct UpdateStackInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<TriggerConfig>,
}

/// List all stacks for the current project
#[tauri::command]
pub async fn list_stacks(state: State<'_, AppState>) -> Result<Vec<Stack>, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let stacks = db.list_stacks(project.id).map_err(|e| e.to_string())?;

    Ok(stacks)
}

/// Get a single stack by ID
#[tauri::command]
pub async fn get_stack(state: State<'_, AppState>, stack_id: String) -> Result<Option<Stack>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

    let stack = db.get_stack(stack_uuid).map_err(|e| e.to_string())?;

    Ok(stack)
}

/// Create a new stack
#[tauri::command]
pub async fn create_stack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    input: CreateStackInput,
) -> Result<Stack, String> {
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    let mut stack = Stack::new(project.id, &input.name, &input.description);
    stack.trigger = input.trigger;
    stack.source_session_id = input
        .source_session_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    db.create_stack(&stack).map_err(|e| e.to_string())?;

    info!("Created stack: {} ({})", stack.name, stack.id);

    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::StackCreated(stack.clone())) {
        warn!("Failed to emit StackCreated event: {}", e);
    }

    Ok(stack)
}

/// Update an existing stack
#[tauri::command]
pub async fn update_stack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    stack_id: String,
    input: UpdateStackInput,
) -> Result<Stack, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

    let mut stack = db
        .get_stack(stack_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Stack not found")?;

    if let Some(name) = input.name {
        stack.name = name;
    }
    if let Some(description) = input.description {
        stack.description = description;
    }
    if let Some(trigger) = input.trigger {
        stack.trigger = trigger;
    }
    stack.updated_at = chrono::Utc::now();

    db.update_stack(&stack).map_err(|e| e.to_string())?;

    info!("Updated stack: {} ({})", stack.name, stack.id);

    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::StackUpdated(stack.clone())) {
        warn!("Failed to emit StackUpdated event: {}", e);
    }

    Ok(stack)
}

/// Delete a stack and all its cards (via CASCADE)
#[tauri::command]
pub async fn delete_stack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    stack_id: String,
) -> Result<(), String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

    // Verify stack exists
    db.get_stack(stack_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Stack not found")?;

    // Delete stack (cards are CASCADE deleted)
    db.delete_stack(stack_uuid).map_err(|e| e.to_string())?;

    info!("Deleted stack: {}", stack_id);

    if let Err(e) = app_handle.emit(
        "leaf-event",
        &LeafEvent::StackDeleted {
            stack_id: stack_uuid,
        },
    ) {
        warn!("Failed to emit StackDeleted event: {}", e);
    }

    Ok(())
}

/// Enable a stack
#[tauri::command]
pub async fn enable_stack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    stack_id: String,
) -> Result<Stack, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

    db.enable_stack(stack_uuid).map_err(|e| e.to_string())?;

    let stack = db
        .get_stack(stack_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Stack not found")?;

    info!("Enabled stack: {} ({})", stack.name, stack.id);

    if let Err(e) = app_handle.emit(
        "leaf-event",
        &LeafEvent::StackEnabled {
            stack_id: stack_uuid,
        },
    ) {
        warn!("Failed to emit StackEnabled event: {}", e);
    }

    Ok(stack)
}

/// Disable a stack
#[tauri::command]
pub async fn disable_stack(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    stack_id: String,
) -> Result<Stack, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

    db.disable_stack(stack_uuid).map_err(|e| e.to_string())?;

    let stack = db
        .get_stack(stack_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Stack not found")?;

    info!("Disabled stack: {} ({})", stack.name, stack.id);

    if let Err(e) = app_handle.emit(
        "leaf-event",
        &LeafEvent::StackDisabled {
            stack_id: stack_uuid,
        },
    ) {
        warn!("Failed to emit StackDisabled event: {}", e);
    }

    Ok(stack)
}
