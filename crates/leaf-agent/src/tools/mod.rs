//! Tool system for the LEAF agent
//!
//! This module provides the infrastructure for defining and executing tools
//! that the LLM can use during conversations.

pub mod filesystem;
pub mod mcp;
pub mod stack;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use leaf_db::Database;
use leaf_mcp::McpClient;
use serde_json::Value;
use tauri::AppHandle;
use uuid::Uuid;

use crate::error::{AgentError, AgentResult};
use crate::provider::ToolDefinition;

/// Context provided to tools when they execute
pub struct ToolContext {
    /// Project root path
    pub project_path: PathBuf,
    /// Project ID
    pub project_id: Uuid,
    /// Database connection
    pub db: Database,
    /// Tauri app handle for emitting events
    pub app_handle: AppHandle,
    /// Session ID for tracking
    pub session_id: Uuid,
    /// MCP client for external tools
    pub mcp_client: Option<Arc<McpClient>>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(
        project_path: PathBuf,
        project_id: Uuid,
        db: Database,
        app_handle: AppHandle,
        session_id: Uuid,
    ) -> Self {
        Self {
            project_path,
            project_id,
            db,
            app_handle,
            session_id,
            mcp_client: None,
        }
    }

    /// Create a new tool context with MCP client
    pub fn with_mcp_client(
        project_path: PathBuf,
        project_id: Uuid,
        db: Database,
        app_handle: AppHandle,
        session_id: Uuid,
        mcp_client: Arc<McpClient>,
    ) -> Self {
        Self {
            project_path,
            project_id,
            db,
            app_handle,
            session_id,
            mcp_client: Some(mcp_client),
        }
    }

    /// Get the .leaf directory path
    pub fn leaf_dir(&self) -> PathBuf {
        self.project_path.join(".leaf")
    }

    /// Get the programs directory path
    pub fn programs_dir(&self) -> PathBuf {
        self.leaf_dir().join("programs")
    }
}

/// Trait for tools that the agent can execute
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the name of the tool
    fn name(&self) -> &str;

    /// Get a description of what the tool does
    fn description(&self) -> &str;

    /// Get the JSON Schema for the tool's parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with the given arguments
    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value>;

    /// Convert to a ToolDefinition for the LLM
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.parameters_schema(),
        }
    }
}

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with the default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register filesystem tools
        registry.register(Arc::new(filesystem::ReadFileTool));
        registry.register(Arc::new(filesystem::WriteFileTool));
        registry.register(Arc::new(filesystem::ListDirectoryTool));
        registry.register(Arc::new(filesystem::CreateFolderTool));
        registry.register(Arc::new(filesystem::DeleteFileTool));
        registry.register(Arc::new(filesystem::MoveFileTool));

        // Register stack tools
        registry.register(Arc::new(stack::ProposeStackTool));
        registry.register(Arc::new(stack::CreateStackNowTool));
        registry.register(Arc::new(stack::AddCardTool));

        // Register MCP tools
        registry.register(Arc::new(mcp::ListMcpToolsTool));
        registry.register(Arc::new(mcp::CallMcpToolTool));

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Get all tool definitions
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.to_definition()).collect()
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        name: &str,
        ctx: &ToolContext,
        args: Value,
    ) -> AgentResult<Value> {
        let tool = self.get(name).ok_or_else(|| {
            AgentError::InvalidToolCall(format!("Unknown tool: {}", name))
        })?;

        tool.execute(ctx, args).await
    }

    /// List all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Helper to create a JSON Schema object type
#[macro_export]
macro_rules! json_schema {
    (
        description: $desc:expr,
        properties: {
            $($name:literal => {
                type: $type:literal,
                description: $prop_desc:expr
                $(, required: $required:literal)?
            }),* $(,)?
        }
    ) => {{
        let mut properties = serde_json::Map::new();
        #[allow(unused_mut)]
        let mut required: Vec<String> = Vec::new();

        $(
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), serde_json::json!($type));
            prop.insert("description".to_string(), serde_json::json!($prop_desc));
            properties.insert($name.to_string(), serde_json::Value::Object(prop));

            $(
                if $required {
                    required.push($name.to_string());
                }
            )?
        )*

        serde_json::json!({
            "type": "object",
            "description": $desc,
            "properties": properties,
            "required": required
        })
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_with_defaults() {
        let registry = ToolRegistry::with_defaults();
        // Filesystem tools
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("write_file").is_some());
        assert!(registry.get("list_directory").is_some());
        assert!(registry.get("create_folder").is_some());
        assert!(registry.get("delete_file").is_some());
        assert!(registry.get("move_file").is_some());
        // Stack tools
        assert!(registry.get("propose_stack").is_some());
        assert!(registry.get("create_stack_now").is_some());
        assert!(registry.get("add_card").is_some());
        // Old card tools should be gone
        assert!(registry.get("propose_card").is_none());
        assert!(registry.get("create_card_now").is_none());
    }

    #[test]
    fn test_tool_definitions() {
        let registry = ToolRegistry::with_defaults();
        let defs = registry.definitions();
        assert!(!defs.is_empty());

        // Check that all definitions have required fields
        for def in defs {
            assert!(!def.name.is_empty());
            assert!(!def.description.is_empty());
        }
    }

    #[test]
    fn test_json_schema_macro() {
        let schema = json_schema!(
            description: "Test schema",
            properties: {
                "name" => {
                    type: "string",
                    description: "The name",
                    required: true
                },
                "count" => {
                    type: "integer",
                    description: "The count"
                }
            }
        );

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&serde_json::json!("name")));
    }
}
