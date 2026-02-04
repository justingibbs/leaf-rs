//! LEAF Agent - LLM agent and code generation for LEAF
//!
//! This crate provides:
//! - Multi-provider LLM abstraction (Anthropic, OpenAI, Google, Ollama)
//! - Tool system for agent capabilities
//! - TypeScript code generation for cards
//! - Agent orchestration and conversation management

pub mod agent;
pub mod codegen;
pub mod error;
pub mod prompts;
pub mod provider;
pub mod tools;

// Re-export main types
pub use agent::{Agent, AgentBuilder, AgentContext};
pub use codegen::generate_card_program;
pub use error::{AgentError, AgentResult};
pub use prompts::system_prompt;
pub use provider::{
    ChatChunk, ChatMessage, ChatResponse, ChatRole, ChatStream, ContentBlockType, LlmProvider,
    ProviderConfig, ProviderType, StopReason, ToolCallRequest, ToolDefinition, ToolResult, Usage,
    create_provider,
};
pub use tools::{Tool, ToolContext, ToolRegistry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exports() {
        // Verify main types are exported
        let _: fn() -> ProviderConfig = ProviderConfig::default;
        let _: fn() -> ToolRegistry = ToolRegistry::new;
    }
}
