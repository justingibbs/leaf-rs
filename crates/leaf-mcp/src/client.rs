//! High-level MCP client
//!
//! This module provides the main `McpClient` interface for managing
//! multiple MCP server connections and routing tool calls.

use std::collections::HashMap;

use leaf_core::McpServer;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{McpError, McpResult};
use crate::protocol::{McpToolInfo, ToolCallResult, ToolContent};
use crate::server::ServerProcess;

/// Summary of a tool from an MCP server
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpToolSummary {
    /// Server name
    pub server_name: String,
    /// Server ID
    pub server_id: Uuid,
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
}

/// High-level MCP client managing multiple server connections
pub struct McpClient {
    /// Connected servers keyed by ID
    servers: RwLock<HashMap<Uuid, ServerProcess>>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
        }
    }

    /// Connect to an MCP server
    pub async fn connect(&self, config: McpServer) -> McpResult<()> {
        let server_id = config.id;
        let server_name = config.name.clone();

        // Check if already connected
        {
            let servers = self.servers.read().await;
            if servers.contains_key(&server_id) {
                info!("MCP server '{}' already connected", server_name);
                return Ok(());
            }
        }

        // Start the server process
        let server = ServerProcess::start(config).await?;

        // Store in map
        {
            let mut servers = self.servers.write().await;
            servers.insert(server_id, server);
        }

        info!("Connected to MCP server: {}", server_name);
        Ok(())
    }

    /// Disconnect from an MCP server
    pub async fn disconnect(&self, server_id: Uuid) -> McpResult<()> {
        let server = {
            let mut servers = self.servers.write().await;
            servers.remove(&server_id)
        };

        if let Some(server) = server {
            let name = server.name().to_string();
            server.stop().await?;
            info!("Disconnected from MCP server: {}", name);
        } else {
            debug!("Server {} not connected, nothing to disconnect", server_id);
        }

        Ok(())
    }

    /// Disconnect from all servers
    pub async fn disconnect_all(&self) {
        let servers: Vec<(Uuid, ServerProcess)> = {
            let mut servers = self.servers.write().await;
            servers.drain().collect()
        };

        for (id, server) in servers {
            let name = server.name().to_string();
            if let Err(e) = server.stop().await {
                warn!("Error stopping MCP server '{}' ({}): {}", name, id, e);
            } else {
                info!("Disconnected from MCP server: {}", name);
            }
        }
    }

    /// List all connected servers
    pub async fn list_servers(&self) -> Vec<(Uuid, String, usize)> {
        let servers = self.servers.read().await;
        servers
            .values()
            .map(|s| (s.id(), s.name().to_string(), s.tools.len()))
            .collect()
    }

    /// Check if a server is connected
    pub async fn is_connected(&self, server_id: Uuid) -> bool {
        let servers = self.servers.read().await;
        servers.contains_key(&server_id)
    }

    /// Get the number of connected servers
    pub async fn connected_count(&self) -> usize {
        let servers = self.servers.read().await;
        servers.len()
    }

    /// List all available tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<McpToolSummary> {
        let servers = self.servers.read().await;
        let mut tools = Vec::new();

        for server in servers.values() {
            for tool in &server.tools {
                tools.push(McpToolSummary {
                    server_name: server.name().to_string(),
                    server_id: server.id(),
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                });
            }
        }

        tools
    }

    /// List tools from a specific server
    pub async fn list_tools_for_server(&self, server_id: Uuid) -> McpResult<Vec<McpToolInfo>> {
        let servers = self.servers.read().await;
        let server = servers
            .get(&server_id)
            .ok_or_else(|| McpError::ServerNotConnected(server_id.to_string()))?;

        Ok(server.tools.clone())
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_id: Uuid,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolCallResult> {
        let servers = self.servers.read().await;
        let server = servers
            .get(&server_id)
            .ok_or_else(|| McpError::ServerNotConnected(server_id.to_string()))?;

        server.call_tool(tool_name, arguments).await
    }

    /// Call a tool by server name
    pub async fn call_tool_by_server_name(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolCallResult> {
        let servers = self.servers.read().await;
        let server = servers
            .values()
            .find(|s| s.name() == server_name)
            .ok_or_else(|| McpError::ServerNotFound(server_name.to_string()))?;

        server.call_tool(tool_name, arguments).await
    }

    /// Find a server by name
    pub async fn find_server_by_name(&self, name: &str) -> Option<Uuid> {
        let servers = self.servers.read().await;
        servers
            .values()
            .find(|s| s.name() == name)
            .map(|s| s.id())
    }

    /// Find which server has a specific tool
    pub async fn find_server_with_tool(&self, tool_name: &str) -> Option<(Uuid, String)> {
        let servers = self.servers.read().await;
        for server in servers.values() {
            if server.has_tool(tool_name) {
                return Some((server.id(), server.name().to_string()));
            }
        }
        None
    }

    /// Get tool info from any connected server
    pub async fn get_tool_info(&self, tool_name: &str) -> Option<(String, McpToolInfo)> {
        let servers = self.servers.read().await;
        for server in servers.values() {
            if let Some(tool) = server.get_tool(tool_name) {
                return Some((server.name().to_string(), tool.clone()));
            }
        }
        None
    }

    /// Get connection status for a server
    pub async fn get_server_status(&self, server_id: Uuid) -> Option<(String, usize)> {
        let servers = self.servers.read().await;
        servers
            .get(&server_id)
            .map(|s| (s.name().to_string(), s.tools.len()))
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to extract text content from tool call result
pub fn extract_text_content(result: &ToolCallResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| match c {
            ToolContent::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_new() {
        let client = McpClient::new();
        assert_eq!(client.connected_count().await, 0);
    }

    #[tokio::test]
    async fn test_list_all_tools_empty() {
        let client = McpClient::new();
        let tools = client.list_all_tools().await;
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tool_summary_serialization() {
        let summary = McpToolSummary {
            server_name: "test".to_string(),
            server_id: Uuid::new_v4(),
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("test_tool"));
    }
}
