//! MCP server process management
//!
//! This module handles spawning and managing MCP server subprocesses,
//! including initialization handshake and tool discovery.

use std::process::Stdio;

use leaf_core::McpServer;
use tokio::process::{Child, Command};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{McpError, McpResult};
use crate::protocol::{
    InitializeParams, InitializeResult, McpToolInfo, ServerCapabilities, ToolCallParams,
    ToolCallResult, ToolsListResult,
};
use crate::transport::StdioTransport;

/// Default timeout for MCP operations in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// A running MCP server process
pub struct ServerProcess {
    /// Server configuration
    pub config: McpServer,
    /// Child process handle
    process: Child,
    /// Transport for communication
    transport: StdioTransport,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Available tools
    pub tools: Vec<McpToolInfo>,
}

impl ServerProcess {
    /// Start a new MCP server process
    pub async fn start(config: McpServer) -> McpResult<Self> {
        info!(
            "Starting MCP server: {} ({})",
            config.name, config.command
        );
        debug!("Server args: {:?}, env: {:?}", config.args, config.env);

        // Spawn the server process
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        let mut process = cmd.spawn().map_err(|e| {
            McpError::ProcessError(format!(
                "Failed to spawn MCP server '{}': {}",
                config.name, e
            ))
        })?;

        // Get stdin/stdout handles
        let stdin = process.stdin.take().ok_or_else(|| {
            McpError::ProcessError("Failed to get stdin handle".to_string())
        })?;
        let stdout = process.stdout.take().ok_or_else(|| {
            McpError::ProcessError("Failed to get stdout handle".to_string())
        })?;

        // Create transport
        let transport = StdioTransport::new(stdin, stdout);

        // Initialize the server
        let init_result = Self::initialize(&transport).await?;
        info!(
            "MCP server '{}' initialized: {} v{}",
            config.name,
            init_result.server_info.name,
            init_result.server_info.version.as_deref().unwrap_or("unknown")
        );

        // List available tools
        let tools = Self::list_tools(&transport).await?;
        info!("MCP server '{}' has {} tools", config.name, tools.len());
        for tool in &tools {
            debug!(
                "  - {}: {}",
                tool.name,
                tool.description.as_deref().unwrap_or("(no description)")
            );
        }

        Ok(Self {
            config,
            process,
            transport,
            capabilities: init_result.capabilities,
            tools,
        })
    }

    /// Perform the MCP initialization handshake
    async fn initialize(transport: &StdioTransport) -> McpResult<InitializeResult> {
        let params = InitializeParams::default();
        let params_value = serde_json::to_value(&params)?;

        let response = transport
            .send_request_with_timeout("initialize", Some(params_value), DEFAULT_TIMEOUT_SECS)
            .await?;

        if let Some(error) = response.error {
            return Err(McpError::JsonRpcError {
                code: error.code,
                message: error.message,
            });
        }

        let result: InitializeResult = serde_json::from_value(
            response.result.ok_or_else(|| {
                McpError::Protocol("Initialize response missing result".to_string())
            })?,
        )?;

        // Send initialized notification
        transport
            .send_notification("notifications/initialized", None)
            .await?;

        Ok(result)
    }

    /// List available tools from the server
    async fn list_tools(transport: &StdioTransport) -> McpResult<Vec<McpToolInfo>> {
        let response = transport
            .send_request_with_timeout("tools/list", None, DEFAULT_TIMEOUT_SECS)
            .await?;

        if let Some(error) = response.error {
            return Err(McpError::JsonRpcError {
                code: error.code,
                message: error.message,
            });
        }

        let result: ToolsListResult = serde_json::from_value(
            response.result.ok_or_else(|| {
                McpError::Protocol("tools/list response missing result".to_string())
            })?,
        )?;

        Ok(result.tools)
    }

    /// Refresh the tools list from the server
    pub async fn refresh_tools(&mut self) -> McpResult<()> {
        self.tools = Self::list_tools(&self.transport).await?;
        Ok(())
    }

    /// Call a tool on the server
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolCallResult> {
        // Verify tool exists
        if !self.tools.iter().any(|t| t.name == name) {
            return Err(McpError::ToolNotFound(format!(
                "Tool '{}' not found on server '{}'",
                name, self.config.name
            )));
        }

        let params = ToolCallParams {
            name: name.to_string(),
            arguments,
        };
        let params_value = serde_json::to_value(&params)?;

        let response = self
            .transport
            .send_request_with_timeout("tools/call", Some(params_value), DEFAULT_TIMEOUT_SECS)
            .await
            .map_err(|e| {
                McpError::ToolExecutionFailed(format!("Failed to call tool '{}': {}", name, e))
            })?;

        if let Some(error) = response.error {
            return Err(McpError::ToolExecutionFailed(format!(
                "Tool '{}' error {}: {}",
                name, error.code, error.message
            )));
        }

        let result: ToolCallResult = serde_json::from_value(
            response.result.ok_or_else(|| {
                McpError::Protocol("tools/call response missing result".to_string())
            })?,
        )?;

        if result.is_error {
            let error_text = result
                .content
                .iter()
                .filter_map(|c| match c {
                    crate::protocol::ToolContent::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(McpError::ToolExecutionFailed(format!(
                "Tool '{}' returned error: {}",
                name, error_text
            )));
        }

        Ok(result)
    }

    /// Stop the server process
    pub async fn stop(mut self) -> McpResult<()> {
        info!("Stopping MCP server: {}", self.config.name);

        // Close transport
        self.transport.close().await;

        // Try graceful shutdown first
        if let Err(e) = self.process.kill().await {
            warn!(
                "Failed to kill MCP server '{}': {}",
                self.config.name, e
            );
        }

        Ok(())
    }

    /// Get the server's name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the server's ID
    pub fn id(&self) -> Uuid {
        self.config.id
    }

    /// Check if a tool is available on this server
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }

    /// Get tool info by name
    pub fn get_tool(&self, name: &str) -> Option<&McpToolInfo> {
        self.tools.iter().find(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let project_id = Uuid::new_v4();
        let server = McpServer::new(project_id, "test-server", "echo");
        assert_eq!(server.name, "test-server");
        assert_eq!(server.command, "echo");
        assert!(server.enabled);
    }
}
