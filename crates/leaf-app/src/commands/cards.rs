//! Card-related Tauri commands

use chrono::Utc;
use leaf_core::{Card, Execution, ExecutionStatus, LeafEvent, ProgramConfig, TriggerConfig};
use leaf_db::{CardQueries, ExecutionQueries};
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Input for creating a new card
#[derive(Debug, serde::Deserialize)]
pub struct CreateCardInput {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub trigger: TriggerConfig,
    #[serde(default)]
    pub program: ProgramConfig,
    pub session_id: Option<String>,
}

/// Input for updating an existing card
#[derive(Debug, serde::Deserialize)]
pub struct UpdateCardInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<TriggerConfig>,
    pub program: Option<ProgramConfig>,
    pub enabled: Option<bool>,
}

/// List all cards for the current project
#[tauri::command]
pub async fn list_cards(state: State<'_, AppState>) -> Result<Vec<Card>, String> {
    debug!("Listing cards");

    let current = state
        .current_project
        .read()
        .map_err(|e| format!("Failed to read project state: {}", e))?;

    let open_project = current
        .as_ref()
        .ok_or_else(|| "No project open".to_string())?;

    let cards = open_project
        .db
        .list_cards(open_project.project.id)
        .map_err(|e| format!("Failed to list cards: {}", e))?;

    debug!("Found {} cards", cards.len());
    Ok(cards)
}

/// Get a single card by ID
#[tauri::command]
pub async fn get_card(state: State<'_, AppState>, card_id: String) -> Result<Option<Card>, String> {
    debug!("Getting card: {}", card_id);

    let id = Uuid::parse_str(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    let card = db.get_card(id).map_err(|e| format!("Failed to get card: {}", e))?;

    Ok(card)
}

/// Create a new card
#[tauri::command]
pub async fn create_card(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateCardInput,
) -> Result<Card, String> {
    debug!("Creating card: {}", input.name);

    let current = state
        .current_project
        .read()
        .map_err(|e| format!("Failed to read project state: {}", e))?;

    let open_project = current
        .as_ref()
        .ok_or_else(|| "No project open".to_string())?;

    // Create the card
    let mut card = Card::new(open_project.project.id, &input.name, &input.description);
    card.trigger = input.trigger;
    card.program = input.program;
    card.session_id = input
        .session_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    // Persist to database
    open_project
        .db
        .create_card(&card)
        .map_err(|e| format!("Failed to create card: {}", e))?;

    info!("Created card: {} ({})", card.name, card.id);

    // Emit event to frontend
    if let Err(e) = app.emit("leaf-event", &LeafEvent::CardCreated(card.clone())) {
        tracing::warn!("Failed to emit card created event: {}", e);
    }

    Ok(card)
}

/// Update an existing card
#[tauri::command]
pub async fn update_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
    input: UpdateCardInput,
) -> Result<Card, String> {
    debug!("Updating card: {}", card_id);

    let id = Uuid::parse_str(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    // Get existing card
    let mut card = db
        .get_card(id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;

    // Apply updates
    if let Some(name) = input.name {
        card.name = name;
    }
    if let Some(description) = input.description {
        card.description = description;
    }
    if let Some(trigger) = input.trigger {
        card.trigger = trigger;
    }
    if let Some(program) = input.program {
        card.program = program;
    }
    if let Some(enabled) = input.enabled {
        card.enabled = enabled;
    }
    card.updated_at = Utc::now();

    // Persist changes
    db.update_card(&card)
        .map_err(|e| format!("Failed to update card: {}", e))?;

    info!("Updated card: {} ({})", card.name, card.id);

    // Emit event to frontend
    if let Err(e) = app.emit("leaf-event", &LeafEvent::CardUpdated(card.clone())) {
        tracing::warn!("Failed to emit card updated event: {}", e);
    }

    Ok(card)
}

/// Delete a card
#[tauri::command]
pub async fn delete_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<(), String> {
    debug!("Deleting card: {}", card_id);

    let id = Uuid::parse_str(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    // Delete from database
    db.delete_card(id)
        .map_err(|e| format!("Failed to delete card: {}", e))?;

    info!("Deleted card: {}", id);

    // Emit event to frontend
    if let Err(e) = app.emit("leaf-event", &LeafEvent::CardDeleted { card_id: id }) {
        tracing::warn!("Failed to emit card deleted event: {}", e);
    }

    Ok(())
}

/// Enable a card
#[tauri::command]
pub async fn enable_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Card, String> {
    debug!("Enabling card: {}", card_id);

    let id = Uuid::parse_str(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    // Get existing card
    let mut card = db
        .get_card(id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;

    // Enable the card
    card.enabled = true;
    card.updated_at = Utc::now();

    // Persist changes
    db.update_card(&card)
        .map_err(|e| format!("Failed to update card: {}", e))?;

    info!("Enabled card: {} ({})", card.name, card.id);

    // Emit event to frontend
    if let Err(e) = app.emit("leaf-event", &LeafEvent::CardEnabled { card_id: id }) {
        tracing::warn!("Failed to emit card enabled event: {}", e);
    }

    Ok(card)
}

/// Disable a card
#[tauri::command]
pub async fn disable_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Card, String> {
    debug!("Disabling card: {}", card_id);

    let id = Uuid::parse_str(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;

    // Get existing card
    let mut card = db
        .get_card(id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;

    // Disable the card
    card.enabled = false;
    card.updated_at = Utc::now();

    // Persist changes
    db.update_card(&card)
        .map_err(|e| format!("Failed to update card: {}", e))?;

    info!("Disabled card: {} ({})", card.name, card.id);

    // Emit event to frontend
    if let Err(e) = app.emit("leaf-event", &LeafEvent::CardDisabled { card_id: id }) {
        tracing::warn!("Failed to emit card disabled event: {}", e);
    }

    Ok(card)
}

/// Trigger a card manually
///
/// This creates a manual event and triggers the card's execution.
/// The execution runs asynchronously in a spawned task.
#[tauri::command]
pub async fn trigger_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Execution, String> {
    debug!("Manually triggering card: {}", card_id);

    let id = Uuid::parse_str(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    let db = state.get_db().map_err(|e| e.to_string())?;
    let project_path = state.get_project_path().map_err(|e| e.to_string())?;

    // Verify card exists and is enabled
    let card = db
        .get_card(id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;

    if !card.enabled {
        return Err("Card is disabled".to_string());
    }

    // Get executor
    let executor = state
        .get_executor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Executor not available. Is Deno installed?".to_string())?;

    info!(
        "Manual trigger requested for card: {} ({})",
        card.name, card.id
    );

    // Create execution record
    let mut execution = Execution::new(card.id, None);
    execution.status = ExecutionStatus::Pending;

    // Persist to database
    db.create_execution(&execution)
        .map_err(|e| format!("Failed to create execution: {}", e))?;

    // Emit execution started event
    if let Err(e) = app.emit("leaf-event", &LeafEvent::ExecutionStarted(execution.clone())) {
        warn!("Failed to emit execution started event: {}", e);
    }

    // Spawn async task to run execution
    let execution_id = execution.id;
    let db_clone = db.clone();
    let app_clone = app.clone();
    let max_retries = card.program.max_retries;

    tokio::spawn(async move {
        run_execution(
            executor,
            card,
            execution_id,
            None, // No event for manual triggers
            project_path,
            db_clone,
            app_clone,
            max_retries,
        )
        .await;
    });

    Ok(execution)
}

/// Run an execution with retry logic
async fn run_execution(
    executor: leaf_executor::Executor,
    card: Card,
    execution_id: Uuid,
    event: Option<leaf_core::Event>,
    project_path: std::path::PathBuf,
    db: leaf_db::Database,
    app: AppHandle,
    max_retries: u32,
) {
    let mut attempt = 1u32;

    loop {
        // Update execution status to running
        if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
            exec.status = ExecutionStatus::Running;
            exec.attempt = attempt;
            if let Err(e) = db.update_execution(&exec) {
                error!("Failed to update execution status: {}", e);
            }
        }

        // Run the execution
        let result = executor
            .execute(&card, event.as_ref(), &project_path)
            .await;

        // Process result
        match result {
            Ok(exec_result) => {
                // Update execution with result
                if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
                    exec.stdout = exec_result.stdout;
                    exec.stderr = exec_result.stderr;
                    exec.exit_code = Some(exec_result.exit_code);
                    exec.duration_ms = Some(exec_result.duration_ms);
                    exec.completed_at = Some(Utc::now());

                    if exec_result.exit_code == 0 {
                        exec.status = ExecutionStatus::Success;
                        info!("Execution {} completed successfully", execution_id);
                    } else {
                        // Check if we should retry
                        if attempt < max_retries {
                            warn!(
                                "Execution {} failed (attempt {}/{}), retrying...",
                                execution_id, attempt, max_retries
                            );
                            attempt += 1;
                            if let Err(e) = db.update_execution(&exec) {
                                error!("Failed to update execution: {}", e);
                            }
                            // Brief delay before retry
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                        exec.status = ExecutionStatus::Failed;
                        error!(
                            "Execution {} failed after {} attempts",
                            execution_id, attempt
                        );
                    }

                    if let Err(e) = db.update_execution(&exec) {
                        error!("Failed to update execution: {}", e);
                    }

                    // Emit completion event
                    if let Err(e) = app.emit(
                        "leaf-event",
                        &LeafEvent::ExecutionCompleted {
                            execution_id,
                            status: exec.status.clone(),
                            exit_code: exec.exit_code,
                        },
                    ) {
                        warn!("Failed to emit execution completed event: {}", e);
                    }
                }
                break;
            }
            Err(leaf_executor::ExecutorError::Timeout(secs)) => {
                // Update execution with timeout status
                if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
                    exec.status = ExecutionStatus::Timeout;
                    exec.stderr = format!("Execution timed out after {} seconds", secs);
                    exec.completed_at = Some(Utc::now());

                    if let Err(e) = db.update_execution(&exec) {
                        error!("Failed to update execution: {}", e);
                    }

                    // Emit completion event
                    if let Err(e) = app.emit(
                        "leaf-event",
                        &LeafEvent::ExecutionCompleted {
                            execution_id,
                            status: ExecutionStatus::Timeout,
                            exit_code: None,
                        },
                    ) {
                        warn!("Failed to emit execution completed event: {}", e);
                    }
                }
                break;
            }
            Err(e) => {
                // Check if we should retry
                if attempt < max_retries {
                    warn!(
                        "Execution {} error (attempt {}/{}): {}, retrying...",
                        execution_id, attempt, max_retries, e
                    );
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }

                // Update execution with failure
                if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
                    exec.status = ExecutionStatus::Failed;
                    exec.stderr = e.to_string();
                    exec.completed_at = Some(Utc::now());

                    if let Err(e) = db.update_execution(&exec) {
                        error!("Failed to update execution: {}", e);
                    }

                    // Emit completion event
                    if let Err(e) = app.emit(
                        "leaf-event",
                        &LeafEvent::ExecutionCompleted {
                            execution_id,
                            status: ExecutionStatus::Failed,
                            exit_code: None,
                        },
                    ) {
                        warn!("Failed to emit execution completed event: {}", e);
                    }
                }
                break;
            }
        }
    }
}
