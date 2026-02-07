//! Card-related Tauri commands (stack-aware)

use leaf_core::{Card, LeafEvent, ProgramConfig};
use leaf_db::CardQueries;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Input for creating a new card within a stack
#[derive(Debug, serde::Deserialize)]
pub struct CreateCardInput {
    pub stack_id: String,
    pub name: String,
    pub description: String,
    pub program: Option<ProgramConfig>,
    pub program_path: Option<String>,
    pub position: Option<i32>,
}

/// Input for updating an existing card
#[derive(Debug, serde::Deserialize)]
pub struct UpdateCardInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub program: Option<ProgramConfig>,
    pub program_path: Option<String>,
    pub position: Option<i32>,
}

/// List all cards for a stack
#[tauri::command]
pub async fn list_cards(
    state: State<'_, AppState>,
    stack_id: String,
) -> Result<Vec<Card>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

    let cards = db
        .list_cards_for_stack(stack_uuid)
        .map_err(|e| e.to_string())?;

    Ok(cards)
}

/// Get a single card by ID
#[tauri::command]
pub async fn get_card(state: State<'_, AppState>, card_id: String) -> Result<Option<Card>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let card_uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;

    let card = db.get_card(card_uuid).map_err(|e| e.to_string())?;

    Ok(card)
}

/// Create a new card within a stack
#[tauri::command]
pub async fn create_card(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    input: CreateCardInput,
) -> Result<Card, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let stack_uuid = Uuid::parse_str(&input.stack_id).map_err(|e| e.to_string())?;

    let mut card = Card::new(stack_uuid, &input.name, &input.description);

    if let Some(program) = input.program {
        card.program = program;
    }
    if let Some(program_path) = input.program_path {
        card.program_path = program_path;
    }
    if let Some(position) = input.position {
        card.position = position;
    }

    db.create_card(&card).map_err(|e| e.to_string())?;

    info!("Created card: {} ({}) in stack {}", card.name, card.id, stack_uuid);

    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::CardCreated(card.clone())) {
        warn!("Failed to emit CardCreated event: {}", e);
    }

    Ok(card)
}

/// Update an existing card
#[tauri::command]
pub async fn update_card(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
    input: UpdateCardInput,
) -> Result<Card, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let card_uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;

    let mut card = db
        .get_card(card_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Card not found")?;

    if let Some(name) = input.name {
        card.name = name;
    }
    if let Some(description) = input.description {
        card.description = description;
    }
    if let Some(program) = input.program {
        card.program = program;
    }
    if let Some(program_path) = input.program_path {
        card.program_path = program_path;
    }
    if let Some(position) = input.position {
        card.position = position;
    }
    card.updated_at = chrono::Utc::now();

    db.update_card(&card).map_err(|e| e.to_string())?;

    info!("Updated card: {} ({})", card.name, card.id);

    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::CardUpdated(card.clone())) {
        warn!("Failed to emit CardUpdated event: {}", e);
    }

    Ok(card)
}

/// Delete a card
#[tauri::command]
pub async fn delete_card(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<(), String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let card_uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;

    let card = db
        .get_card(card_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Card not found")?;

    let stack_id = card.stack_id;

    db.delete_card(card_uuid).map_err(|e| e.to_string())?;

    info!("Deleted card: {} ({})", card.name, card_id);

    if let Err(e) = app_handle.emit(
        "leaf-event",
        &LeafEvent::CardDeleted {
            card_id: card_uuid,
            stack_id,
        },
    ) {
        warn!("Failed to emit CardDeleted event: {}", e);
    }

    Ok(())
}

/// Enable a card
#[tauri::command]
pub async fn enable_card(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Card, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let card_uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;

    let mut card = db
        .get_card(card_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Card not found")?;

    card.enabled = true;
    card.updated_at = chrono::Utc::now();

    db.update_card(&card).map_err(|e| e.to_string())?;

    info!("Enabled card: {} ({})", card.name, card.id);

    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::CardUpdated(card.clone())) {
        warn!("Failed to emit CardUpdated event: {}", e);
    }

    Ok(card)
}

/// Disable a card
#[tauri::command]
pub async fn disable_card(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Card, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let card_uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;

    let mut card = db
        .get_card(card_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Card not found")?;

    card.enabled = false;
    card.updated_at = chrono::Utc::now();

    db.update_card(&card).map_err(|e| e.to_string())?;

    info!("Disabled card: {} ({})", card.name, card.id);

    if let Err(e) = app_handle.emit("leaf-event", &LeafEvent::CardUpdated(card.clone())) {
        warn!("Failed to emit CardUpdated event: {}", e);
    }

    Ok(card)
}

/// Trigger a card manually
///
/// Looks up the card's stack and runs the full stack execution pipeline.
#[tauri::command]
pub async fn trigger_card(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<serde_json::Value, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let card_uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;

    let card = db
        .get_card(card_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Card not found")?;

    let executor = state
        .get_executor()
        .map_err(|e| e.to_string())?
        .ok_or("Executor not available (Deno not found)")?;

    let project_path = state.get_project_path().map_err(|e| e.to_string())?;

    info!("Triggering card '{}' (stack {})", card.name, card.stack_id);

    let stack_exec = crate::execution::run_stack(
        card.stack_id,
        None,
        None,
        &db,
        &executor,
        &app_handle,
        &project_path,
    )
    .await?;

    serde_json::to_value(&stack_exec).map_err(|e| e.to_string())
}
