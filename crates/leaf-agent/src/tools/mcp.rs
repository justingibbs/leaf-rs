//! MCP (Model Context Protocol) tools for the LEAF agent
//!
//! These tools allow the agent to interact with MCP servers.
//! This is a stub implementation for Phase 6.

use async_trait::async_trait;
use serde_json::Value;

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

    async fn execute(&self, _ctx: &ToolContext, _args: Value) -> AgentResult<Value> {
        // Stub implementation - will be implemented in Phase 6
        Err(AgentError::ToolError(
            "MCP integration not yet implemented. This feature will be available in a future update.".to_string(),
        ))
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

    async fn execute(&self, _ctx: &ToolContext, _args: Value) -> AgentResult<Value> {
        // Stub implementation - will be implemented in Phase 6
        Err(AgentError::ToolError(
            "MCP integration not yet implemented. This feature will be available in a future update.".to_string(),
        ))
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
    }
}
