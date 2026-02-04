//! MCP server management Tauri commands

use std::sync::Arc;

use leaf_core::{LeafEvent, McpServer};
use leaf_db::McpServerQueries;
use leaf_mcp::{McpClient, McpToolSummary};
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Input for creating a new MCP server
#[derive(Debug, serde::Deserialize)]
pub struct CreateMcpServerInput {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// Response for test_mcp_server command
#[derive(Debug, serde::Serialize)]
pub struct McpServerStatus {
    pub connected: bool,
    pub tool_count: usize,
}

/// List all MCP servers for the current project
#[tauri::command]
pub async fn list_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServer>, String> {
    debug!("Listing MCP servers");

    let (db, project_id) = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        let open_project = current
            .as_ref()
            .ok_or_else(|| "No project open".to_string())?;

        (open_project.db.clone(), open_project.project.id)
    };

    let servers = db
        .list_mcp_servers(project_id)
        .map_err(|e| format!("Failed to list MCP servers: {}", e))?;

    debug!("Found {} MCP servers", servers.len());
    Ok(servers)
}

/// Add a new MCP server
#[tauri::command]
pub async fn add_mcp_server(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateMcpServerInput,
) -> Result<McpServer, String> {
    debug!("Adding MCP server: {}", input.name);

    let (db, project_id, mcp_client) = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        let open_project = current
            .as_ref()
            .ok_or_else(|| "No project open".to_string())?;

        (
            open_project.db.clone(),
            open_project.project.id,
            open_project.mcp_client.clone(),
        )
    };

    // Create the server config
    let mut server = McpServer::new(project_id, &input.name, &input.command);
    server.args = input.args;
    server.env = input.env;

    // Persist to database
    db.create_mcp_server(&server)
        .map_err(|e| format!("Failed to create MCP server: {}", e))?;

    info!("Added MCP server: {} ({})", server.name, server.id);

    // Try to connect if enabled
    if server.enabled {
        if let Some(ref mcp_client) = mcp_client {
            match mcp_client.connect(server.clone()).await {
                Ok(()) => {
                    let tool_count = mcp_client
                        .get_server_status(server.id)
                        .await
                        .map(|(_, count)| count)
                        .unwrap_or(0);

                    let _ = app.emit(
                        "leaf-event",
                        &LeafEvent::McpServerConnected {
                            server_id: server.id,
                            server_name: server.name.clone(),
                            tool_count,
                        },
                    );
                }
                Err(e) => {
                    warn!("Failed to connect to MCP server '{}': {}", server.name, e);
                    let _ = app.emit(
                        "leaf-event",
                        &LeafEvent::McpServerError {
                            server_id: server.id,
                            server_name: server.name.clone(),
                            error: e.to_string(),
                        },
                    );
                }
            }
        }
    }

    Ok(server)
}

/// Remove an MCP server
#[tauri::command]
pub async fn remove_mcp_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<(), String> {
    debug!("Removing MCP server: {}", server_id);

    let id = Uuid::parse_str(&server_id).map_err(|e| format!("Invalid server ID: {}", e))?;

    let (db, mcp_client, server_name) = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        let open_project = current
            .as_ref()
            .ok_or_else(|| "No project open".to_string())?;

        // Get server info before deleting
        let server = open_project
            .db
            .get_mcp_server(id)
            .map_err(|e| format!("Failed to get MCP server: {}", e))?;

        let server_name = server.as_ref().map(|s| s.name.clone()).unwrap_or_default();

        (
            open_project.db.clone(),
            open_project.mcp_client.clone(),
            server_name,
        )
    };

    // Disconnect if connected
    if let Some(ref mcp_client) = mcp_client {
        if mcp_client.is_connected(id).await {
            if let Err(e) = mcp_client.disconnect(id).await {
                warn!("Error disconnecting MCP server '{}': {}", server_name, e);
            }

            let _ = app.emit(
                "leaf-event",
                &LeafEvent::McpServerDisconnected {
                    server_id: id,
                    server_name: server_name.clone(),
                },
            );
        }
    }

    // Delete from database
    db.delete_mcp_server(id)
        .map_err(|e| format!("Failed to delete MCP server: {}", e))?;

    info!("Removed MCP server: {} ({})", server_name, id);
    Ok(())
}

/// Enable an MCP server (and connect)
#[tauri::command]
pub async fn enable_mcp_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<McpServer, String> {
    debug!("Enabling MCP server: {}", server_id);

    let id = Uuid::parse_str(&server_id).map_err(|e| format!("Invalid server ID: {}", e))?;

    let (db, mcp_client) = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        let open_project = current
            .as_ref()
            .ok_or_else(|| "No project open".to_string())?;

        (open_project.db.clone(), open_project.mcp_client.clone())
    };

    // Enable in database
    db.enable_mcp_server(id)
        .map_err(|e| format!("Failed to enable MCP server: {}", e))?;

    // Get updated server
    let server = db
        .get_mcp_server(id)
        .map_err(|e| format!("Failed to get MCP server: {}", e))?
        .ok_or_else(|| "MCP server not found".to_string())?;

    info!("Enabled MCP server: {} ({})", server.name, server.id);

    // Try to connect
    if let Some(ref mcp_client) = mcp_client {
        match mcp_client.connect(server.clone()).await {
            Ok(()) => {
                let tool_count = mcp_client
                    .get_server_status(server.id)
                    .await
                    .map(|(_, count)| count)
                    .unwrap_or(0);

                let _ = app.emit(
                    "leaf-event",
                    &LeafEvent::McpServerConnected {
                        server_id: server.id,
                        server_name: server.name.clone(),
                        tool_count,
                    },
                );
            }
            Err(e) => {
                warn!("Failed to connect to MCP server '{}': {}", server.name, e);
                let _ = app.emit(
                    "leaf-event",
                    &LeafEvent::McpServerError {
                        server_id: server.id,
                        server_name: server.name.clone(),
                        error: e.to_string(),
                    },
                );
            }
        }
    }

    Ok(server)
}

/// Disable an MCP server (and disconnect)
#[tauri::command]
pub async fn disable_mcp_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<McpServer, String> {
    debug!("Disabling MCP server: {}", server_id);

    let id = Uuid::parse_str(&server_id).map_err(|e| format!("Invalid server ID: {}", e))?;

    let (db, mcp_client, server_name) = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        let open_project = current
            .as_ref()
            .ok_or_else(|| "No project open".to_string())?;

        // Get server info
        let server = open_project
            .db
            .get_mcp_server(id)
            .map_err(|e| format!("Failed to get MCP server: {}", e))?
            .ok_or_else(|| "MCP server not found".to_string())?;

        (
            open_project.db.clone(),
            open_project.mcp_client.clone(),
            server.name,
        )
    };

    // Disconnect if connected
    if let Some(ref mcp_client) = mcp_client {
        if mcp_client.is_connected(id).await {
            if let Err(e) = mcp_client.disconnect(id).await {
                warn!("Error disconnecting MCP server '{}': {}", server_name, e);
            }

            let _ = app.emit(
                "leaf-event",
                &LeafEvent::McpServerDisconnected {
                    server_id: id,
                    server_name: server_name.clone(),
                },
            );
        }
    }

    // Disable in database
    db.disable_mcp_server(id)
        .map_err(|e| format!("Failed to disable MCP server: {}", e))?;

    // Get updated server
    let updated_server = db
        .get_mcp_server(id)
        .map_err(|e| format!("Failed to get MCP server: {}", e))?
        .ok_or_else(|| "MCP server not found".to_string())?;

    info!(
        "Disabled MCP server: {} ({})",
        updated_server.name, updated_server.id
    );
    Ok(updated_server)
}

/// List all tools from connected MCP servers
#[tauri::command]
pub async fn list_mcp_tools(state: State<'_, AppState>) -> Result<Vec<McpToolSummary>, String> {
    debug!("Listing MCP tools");

    let mcp_client: Option<Arc<McpClient>> = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        current
            .as_ref()
            .and_then(|p| p.mcp_client.clone())
    };

    let tools = if let Some(ref mcp_client) = mcp_client {
        mcp_client.list_all_tools().await
    } else {
        Vec::new()
    };

    debug!("Found {} MCP tools", tools.len());
    Ok(tools)
}

/// Test MCP server connection status
#[tauri::command]
pub async fn test_mcp_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<McpServerStatus, String> {
    debug!("Testing MCP server: {}", server_id);

    let id = Uuid::parse_str(&server_id).map_err(|e| format!("Invalid server ID: {}", e))?;

    let mcp_client: Option<Arc<McpClient>> = {
        let current = state
            .current_project
            .read()
            .map_err(|e| format!("Failed to read project state: {}", e))?;

        current
            .as_ref()
            .and_then(|p| p.mcp_client.clone())
    };

    let status = if let Some(ref mcp_client) = mcp_client {
        if let Some((_, tool_count)) = mcp_client.get_server_status(id).await {
            McpServerStatus {
                connected: true,
                tool_count,
            }
        } else {
            McpServerStatus {
                connected: false,
                tool_count: 0,
            }
        }
    } else {
        McpServerStatus {
            connected: false,
            tool_count: 0,
        }
    };

    Ok(status)
}
