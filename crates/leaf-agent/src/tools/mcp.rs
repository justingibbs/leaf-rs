//! MCP (Model Context Protocol) tools for the LEAF agent
//!
//! These tools allow the agent to interact with MCP servers,
//! listing available tools and calling them.

use async_trait::async_trait;
use leaf_core::LeafEvent;
use leaf_mcp::extract_text_content;
use serde_json::Value;
use tauri::Emitter;
use tracing::{debug, info};

use super::{Tool, ToolContext};
use crate::error::{AgentError, AgentResult};
use crate::json_schema;

/// Tool for listing available MCP tools
pub struct ListMcpToolsTool;

#[async_trait]
impl Tool for ListMcpToolsTool {
    fn name(&self) -> &str {
        "list_mcp_tools"
    }

    fn description(&self) -> &str {
        "List all available tools from connected MCP servers. This allows access to external tools and data sources."
    }

    fn parameters_schema(&self) -> Value {
        json_schema!(
            description: "List available MCP tools",
            properties: {
                "server_name" => {
                    type: "string",
                    description: "Optional: filter by specific MCP server name"
                }
            }
        )
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let mcp_client = ctx.mcp_client.as_ref().ok_or_else(|| {
            AgentError::ToolError("MCP client not available".to_string())
        })?;

        let server_name_filter = args
            .get("server_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tools = mcp_client.list_all_tools().await;

        // Filter by server name if specified
        let filtered_tools: Vec<_> = if let Some(ref name) = server_name_filter {
            tools.into_iter().filter(|t| &t.server_name == name).collect()
        } else {
            tools
        };

        if filtered_tools.is_empty() {
            let msg = if let Some(name) = server_name_filter {
                format!("No MCP tools available from server '{}'", name)
            } else {
                "No MCP tools available. No MCP servers are connected.".to_string()
            };
            return Ok(serde_json::json!({
                "message": msg,
                "tools": []
            }));
        }

        // Format tools for display
        let tool_list: Vec<Value> = filtered_tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "server": t.server_name,
                    "name": t.name,
                    "description": t.description.as_deref().unwrap_or("No description")
                })
            })
            .collect();

        info!(
            "Listed {} MCP tools{}",
            tool_list.len(),
            server_name_filter.map(|n| format!(" from '{}'", n)).unwrap_or_default()
        );

        Ok(serde_json::json!({
            "message": format!("Found {} MCP tools available", tool_list.len()),
            "tools": tool_list
        }))
    }
}

/// Tool for calling an MCP tool
pub struct CallMcpToolTool;

#[async_trait]
impl Tool for CallMcpToolTool {
    fn name(&self) -> &str {
        "call_mcp_tool"
    }

    fn description(&self) -> &str {
        "Call a tool from a connected MCP server. Use list_mcp_tools first to see available tools."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Call an MCP tool",
            "properties": {
                "server_name": {
                    "type": "string",
                    "description": "Name of the MCP server"
                },
                "tool_name": {
                    "type": "string",
                    "description": "Name of the tool to call"
                },
                "arguments": {
                    "type": "object",
                    "description": "Arguments to pass to the tool"
                }
            },
            "required": ["server_name", "tool_name"]
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let mcp_client = ctx.mcp_client.as_ref().ok_or_else(|| {
            AgentError::ToolError("MCP client not available".to_string())
        })?;

        let server_name = args
            .get("server_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::InvalidToolCall("server_name is required".to_string()))?;

        let tool_name = args
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::InvalidToolCall("tool_name is required".to_string()))?;

        let arguments = args.get("arguments").cloned();

        debug!(
            "Calling MCP tool '{}' on server '{}' with args: {:?}",
            tool_name, server_name, arguments
        );

        // Emit tool call event
        let _ = ctx.app_handle.emit(
            "leaf-event",
            &LeafEvent::McpToolCalled {
                session_id: ctx.session_id,
                server_name: server_name.to_string(),
                tool_name: tool_name.to_string(),
            },
        );

        // Call the tool
        let result = mcp_client
            .call_tool_by_server_name(server_name, tool_name, arguments)
            .await
            .map_err(|e| AgentError::ToolError(e.to_string()))?;

        // Extract text content from result
        let text_content = extract_text_content(&result);

        // Emit result event
        let _ = ctx.app_handle.emit(
            "leaf-event",
            &LeafEvent::McpToolResult {
                session_id: ctx.session_id,
                server_name: server_name.to_string(),
                tool_name: tool_name.to_string(),
                success: !result.is_error,
            },
        );

        info!(
            "MCP tool '{}' on server '{}' completed successfully",
            tool_name, server_name
        );

        Ok(serde_json::json!({
            "success": !result.is_error,
            "content": text_content,
            "raw_content": result.content
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_mcp_tools_definition() {
        let tool = ListMcpToolsTool;
        assert_eq!(tool.name(), "list_mcp_tools");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_call_mcp_tool_definition() {
        let tool = CallMcpToolTool;
        assert_eq!(tool.name(), "call_mcp_tool");
        assert!(!tool.description().is_empty());

        let schema = tool.parameters_schema();
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("server_name")));
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("tool_name")));
    }
}
