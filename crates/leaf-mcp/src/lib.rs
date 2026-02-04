//! LEAF MCP - Model Context Protocol client for LEAF
//!
//! This crate provides MCP (Model Context Protocol) integration:
//! - Server process management
//! - Tool discovery and invocation
//! - Resource access
//! - Prompt templates
//!
//! # Example
//!
//! ```no_run
//! use leaf_mcp::McpClient;
//! use leaf_core::McpServer;
//! use uuid::Uuid;
//!
//! # async fn example() -> leaf_mcp::McpResult<()> {
//! let client = McpClient::new();
//!
//! // Create server config
//! let config = McpServer::new(
//!     Uuid::new_v4(),
//!     "filesystem",
//!     "npx",
//! );
//!
//! // Connect to server
//! client.connect(config).await?;
//!
//! // List available tools
//! let tools = client.list_all_tools().await;
//! for tool in tools {
//!     println!("Tool: {} - {:?}", tool.name, tool.description);
//! }
//!
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod protocol;
pub mod server;
pub mod transport;

// Re-export main types
pub use client::{extract_text_content, McpClient, McpToolSummary};
pub use error::{McpError, McpResult};
pub use protocol::{McpToolInfo, ToolCallResult, ToolContent};
pub use server::ServerProcess;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = McpClient::new();
    }
}
