//! Card-related Tauri commands (stubbed for Phase A)
//!
//! These commands are placeholders that maintain the Tauri IPC interface.
//! They will be rewritten in Phase B with Stack-aware logic.

use leaf_core::Card;
use tauri::{AppHandle, State};
use tracing::debug;

use crate::state::AppState;

/// Input for creating a new card (stub)
#[derive(Debug, serde::Deserialize)]
pub struct CreateCardInput {
    pub name: String,
    pub description: String,
}

/// Input for updating an existing card (stub)
#[derive(Debug, serde::Deserialize)]
pub struct UpdateCardInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

/// List all cards for the current project
/// TODO: Phase B - rewrite to list cards via stacks
#[tauri::command]
pub async fn list_cards(state: State<'_, AppState>) -> Result<Vec<Card>, String> {
    debug!("list_cards: stubbed for Phase A");
    let _ = state;
    Ok(Vec::new())
}

/// Get a single card by ID
/// TODO: Phase B - rewrite with stack-aware logic
#[tauri::command]
pub async fn get_card(state: State<'_, AppState>, card_id: String) -> Result<Option<Card>, String> {
    debug!("get_card: stubbed for Phase A, card_id={}", card_id);
    let _ = state;
    Ok(None)
}

/// Create a new card
/// TODO: Phase B - rewrite to create card within a stack
#[tauri::command]
pub async fn create_card(
    _app: AppHandle,
    state: State<'_, AppState>,
    input: CreateCardInput,
) -> Result<Card, String> {
    debug!("create_card: stubbed for Phase A, name={}", input.name);
    let _ = state;
    Err("Card creation temporarily disabled during Stack migration".to_string())
}

/// Update an existing card
/// TODO: Phase B - rewrite with stack-aware logic
#[tauri::command]
pub async fn update_card(
    _app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
    input: UpdateCardInput,
) -> Result<Card, String> {
    debug!("update_card: stubbed for Phase A, card_id={}", card_id);
    let _ = (state, input);
    Err("Card update temporarily disabled during Stack migration".to_string())
}

/// Delete a card
/// TODO: Phase B - rewrite with stack-aware logic
#[tauri::command]
pub async fn delete_card(
    _app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<(), String> {
    debug!("delete_card: stubbed for Phase A, card_id={}", card_id);
    let _ = state;
    Err("Card deletion temporarily disabled during Stack migration".to_string())
}

/// Enable a card
/// TODO: Phase B - rewrite with stack-aware logic
#[tauri::command]
pub async fn enable_card(
    _app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Card, String> {
    debug!("enable_card: stubbed for Phase A, card_id={}", card_id);
    let _ = state;
    Err("Card enable temporarily disabled during Stack migration".to_string())
}

/// Disable a card
/// TODO: Phase B - rewrite with stack-aware logic
#[tauri::command]
pub async fn disable_card(
    _app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Card, String> {
    debug!("disable_card: stubbed for Phase A, card_id={}", card_id);
    let _ = state;
    Err("Card disable temporarily disabled during Stack migration".to_string())
}

/// Trigger a card manually
/// TODO: Phase C - rewrite with stack execution pipeline
#[tauri::command]
pub async fn trigger_card(
    _app: AppHandle,
    state: State<'_, AppState>,
    card_id: String,
) -> Result<serde_json::Value, String> {
    debug!("trigger_card: stubbed for Phase A, card_id={}", card_id);
    let _ = state;
    Err("Card trigger temporarily disabled during Stack migration".to_string())
}
