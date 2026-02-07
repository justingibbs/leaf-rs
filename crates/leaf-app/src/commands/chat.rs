//! Chat Tauri commands for interacting with the LEAF agent

use leaf_agent::{Agent, AgentContext, ProviderConfig, ProviderType};
use leaf_core::{LeafEvent, Message, MessageRole};
use leaf_db::{MessageQueries, SessionQueries};
use tauri::{AppHandle, Emitter, State};
use tracing::{error, info};
use uuid::Uuid;

use crate::state::AppState;

/// Get messages for a chat session
#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let messages = db
        .list_messages(session_uuid)
        .map_err(|e| e.to_string())?;

    Ok(messages)
}

/// Configuration for the chat agent
#[derive(Debug, serde::Deserialize)]
pub struct AgentConfig {
    /// API key for the provider
    pub api_key: String,
    /// Provider type (defaults to anthropic)
    #[serde(default)]
    pub provider: Option<String>,
    /// Model to use (defaults to claude-sonnet-4-20250514)
    #[serde(default)]
    pub model: Option<String>,
}

/// Send a message to the chat agent
///
/// This command:
/// 1. Saves the user message to the database
/// 2. Spawns an async task to process the message with the agent
/// 3. Returns immediately with the user message ID
/// 4. The agent response is emitted via Tauri events as it streams
#[tauri::command]
pub async fn send_message(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    content: String,
    agent_config: AgentConfig,
) -> Result<Message, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    // Verify session exists
    let session = db
        .get_session(session_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Session not found")?;

    // Get project info
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let project_path = state.get_project_path().map_err(|e| e.to_string())?;

    // Create and save the user message
    let user_message = Message::new(session_uuid, MessageRole::User, &content);
    db.create_message(&user_message)
        .map_err(|e| e.to_string())?;

    info!(
        "User message saved: {} in session {}",
        user_message.id, session_id
    );

    // Note: We do NOT emit a MessageReceived event for the user message here.
    // The user message is returned directly to the frontend via the command response,
    // and the frontend adds it to state. Emitting an event would cause duplication.

    // Clone what we need for the async task
    let app_handle_clone = app_handle.clone();
    let db_clone = db.clone();
    let content_clone = content.clone();

    // Spawn async task to process with agent
    tokio::spawn(async move {
        // Build provider config
        let provider_type = match agent_config.provider.as_deref() {
            Some("openai") => ProviderType::OpenAI,
            Some("google") => ProviderType::Google,
            Some("ollama") => ProviderType::Ollama,
            _ => ProviderType::Anthropic,
        };

        let provider_config = ProviderConfig {
            provider: provider_type,
            api_key: Some(agent_config.api_key),
            model: agent_config
                .model
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            ..Default::default()
        };

        // Create the agent
        let agent = match Agent::new(provider_config) {
            Ok(agent) => agent,
            Err(e) => {
                error!("Failed to create agent: {}", e);
                emit_error(&app_handle_clone, session_uuid, &e.to_string());
                return;
            }
        };

        // Create agent context
        let ctx = AgentContext {
            project_path,
            project_id: project.id,
            db: db_clone.clone(),
            app_handle: app_handle_clone.clone(),
            session,
        };

        // Get agent response with streaming
        match agent.respond_streaming(&ctx, &content_clone).await {
            Ok(response_message) => {
                // Save the assistant message
                if let Err(e) = db_clone.create_message(&response_message) {
                    error!("Failed to save assistant message: {}", e);
                } else {
                    info!(
                        "Assistant message saved: {} in session {}",
                        response_message.id, session_uuid
                    );
                }

                // Final message event is already emitted by respond_streaming
            }
            Err(e) => {
                error!("Agent error: {}", e);
                emit_error(&app_handle_clone, session_uuid, &e.to_string());
            }
        }
    });

    Ok(user_message)
}

/// Send a message and wait for the complete response (non-streaming)
///
/// This is useful for programmatic use where you want the full response.
#[tauri::command]
pub async fn send_message_sync(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    content: String,
    agent_config: AgentConfig,
) -> Result<Message, String> {
    let db = state.get_db().map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    // Verify session exists
    let session = db
        .get_session(session_uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Session not found")?;

    // Get project info
    let project = state
        .get_current_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project open")?;

    let project_path = state.get_project_path().map_err(|e| e.to_string())?;

    // Create and save the user message
    let user_message = Message::new(session_uuid, MessageRole::User, &content);
    db.create_message(&user_message)
        .map_err(|e| e.to_string())?;

    // Build provider config
    let provider_type = match agent_config.provider.as_deref() {
        Some("openai") => ProviderType::OpenAI,
        Some("google") => ProviderType::Google,
        Some("ollama") => ProviderType::Ollama,
        _ => ProviderType::Anthropic,
    };

    let provider_config = ProviderConfig {
        provider: provider_type,
        api_key: Some(agent_config.api_key),
        model: agent_config
            .model
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        ..Default::default()
    };

    // Create the agent
    let agent = Agent::new(provider_config).map_err(|e| e.to_string())?;

    // Create agent context
    let ctx = AgentContext {
        project_path,
        project_id: project.id,
        db: db.clone(),
        app_handle: app_handle.clone(),
        session,
    };

    // Get agent response (non-streaming)
    let response_message = agent
        .respond(&ctx, &content)
        .await
        .map_err(|e| e.to_string())?;

    // Save the assistant message
    db.create_message(&response_message)
        .map_err(|e| e.to_string())?;

    info!(
        "Assistant message saved: {} in session {}",
        response_message.id, session_uuid
    );

    Ok(response_message)
}

/// Helper to emit an error event
fn emit_error(app_handle: &AppHandle, session_id: Uuid, message: &str) {
    if let Err(e) = app_handle.emit(
        "leaf-event",
        &LeafEvent::Error {
            context: format!("chat:{}", session_id),
            message: message.to_string(),
        },
    ) {
        error!("Failed to emit error event: {}", e);
    }
}
