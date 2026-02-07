//! Agent orchestration for LEAF
//!
//! This module provides the main Agent struct that coordinates between
//! the LLM provider, tools, and chat sessions.

use std::path::PathBuf;

use chrono::Utc;
use futures::StreamExt;
use leaf_core::{ChatSession, LeafEvent, Message, MessageRole, ToolCall};
use leaf_db::{Database, MessageQueries};
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AgentError, AgentResult};
use crate::prompts;
use crate::provider::{
    ChatChunk, ChatMessage, ContentBlockType, LlmProvider, ProviderConfig, StopReason,
    ToolCallRequest, create_provider,
};
use crate::tools::{ToolContext, ToolRegistry};

/// The LEAF agent that handles conversations and tool execution
pub struct Agent {
    provider: Box<dyn LlmProvider>,
    tools: ToolRegistry,
    system_prompt: String,
}

/// Context for an agent request
pub struct AgentContext {
    /// Project path
    pub project_path: PathBuf,
    /// Project ID
    pub project_id: Uuid,
    /// Database connection
    pub db: Database,
    /// Tauri app handle
    pub app_handle: AppHandle,
    /// Current session
    pub session: ChatSession,
}

impl Agent {
    /// Create a new agent with the given provider configuration
    pub fn new(config: ProviderConfig) -> AgentResult<Self> {
        let provider = create_provider(config)?;
        let tools = ToolRegistry::with_defaults();
        let system_prompt = prompts::system_prompt();

        info!("Agent initialized with provider: {:?}", provider.provider_type());

        Ok(Self {
            provider,
            tools,
            system_prompt,
        })
    }

    /// Create an agent with a custom tool registry
    pub fn with_tools(config: ProviderConfig, tools: ToolRegistry) -> AgentResult<Self> {
        let provider = create_provider(config)?;
        let system_prompt = prompts::system_prompt();

        Ok(Self {
            provider,
            tools,
            system_prompt,
        })
    }

    /// Get tool definitions for the LLM
    pub fn tool_definitions(&self) -> Vec<crate::provider::ToolDefinition> {
        self.tools.definitions()
    }

    /// Process a user message and generate a response
    ///
    /// This is the main entry point for agent interactions. It:
    /// 1. Loads the conversation history
    /// 2. Sends the message to the LLM
    /// 3. Handles any tool calls
    /// 4. Returns the final response
    pub async fn respond(&self, ctx: &AgentContext, user_message: &str) -> AgentResult<Message> {
        info!(
            "Agent responding to message in session {}",
            ctx.session.id
        );

        // Load conversation history
        let history = ctx.db.list_messages(ctx.session.id)?;

        // Build messages for the LLM
        let mut messages = vec![ChatMessage::system(&self.system_prompt)];

        // Add history
        for msg in &history {
            let chat_msg = self.message_to_chat_message(msg);
            messages.push(chat_msg);
        }

        // Add the new user message
        messages.push(ChatMessage::user(user_message));

        // Get tool definitions
        let tools = self.tool_definitions();
        let tools_ref: Vec<_> = tools.iter().collect();

        // Create tool context for this request
        let tool_ctx = ToolContext::new(
            ctx.project_path.clone(),
            ctx.project_id,
            ctx.db.clone(),
            ctx.app_handle.clone(),
            ctx.session.id,
        );

        // Run the conversation loop
        let mut response_text = String::new();
        let mut all_tool_calls: Vec<ToolCall> = Vec::new();

        loop {
            // Emit thinking event
            if let Err(e) = ctx.app_handle.emit(
                "leaf-event",
                &LeafEvent::AgentThinking {
                    session_id: ctx.session.id,
                },
            ) {
                warn!("Failed to emit AgentThinking event: {}", e);
            }

            // Call the LLM
            let response = self
                .provider
                .chat(&messages, Some(&tools_ref.iter().map(|t| (*t).clone()).collect::<Vec<_>>()))
                .await?;

            debug!(
                "LLM response: stop_reason={:?}, tool_calls={}",
                response.stop_reason,
                response.tool_calls.len()
            );

            // Accumulate text
            if !response.content.is_empty() {
                response_text.push_str(&response.content);
            }

            // Check if we need to handle tool calls
            if response.stop_reason == StopReason::ToolUse && !response.tool_calls.is_empty() {
                // Add assistant message with tool calls
                let mut assistant_msg = ChatMessage::assistant(&response.content);
                assistant_msg.tool_calls = response.tool_calls.clone();
                messages.push(assistant_msg);

                // Execute each tool call
                for tool_call in &response.tool_calls {
                    info!("Executing tool: {}", tool_call.name);

                    // Emit tool call event
                    if let Err(e) = ctx.app_handle.emit(
                        "leaf-event",
                        &LeafEvent::AgentToolCall {
                            session_id: ctx.session.id,
                            tool_name: tool_call.name.clone(),
                            arguments: tool_call.arguments.clone(),
                        },
                    ) {
                        warn!("Failed to emit AgentToolCall event: {}", e);
                    }

                    // Execute the tool
                    let result = self
                        .tools
                        .execute(&tool_call.name, &tool_ctx, tool_call.arguments.clone())
                        .await;

                    match result {
                        Ok(value) => {
                            debug!("Tool {} succeeded: {:?}", tool_call.name, value);
                            messages.push(ChatMessage::tool_result(&tool_call.id, value.clone()));

                            all_tool_calls.push(ToolCall {
                                id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                arguments: tool_call.arguments.clone(),
                                result: Some(value),
                            });
                        }
                        Err(e) => {
                            error!("Tool {} failed: {}", tool_call.name, e);
                            messages.push(ChatMessage::tool_error(&tool_call.id, e.to_string()));

                            all_tool_calls.push(ToolCall {
                                id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                arguments: tool_call.arguments.clone(),
                                result: Some(serde_json::json!({ "error": e.to_string() })),
                            });
                        }
                    }
                }

                // Continue the loop to get the next response
                continue;
            }

            // No more tool calls, we're done
            break;
        }

        // Create the final assistant message
        let mut final_message = Message::new(ctx.session.id, MessageRole::Assistant, &response_text);
        final_message.tool_calls = all_tool_calls;

        // Emit message received event
        if let Err(e) = ctx.app_handle.emit(
            "leaf-event",
            &LeafEvent::MessageReceived(final_message.clone()),
        ) {
            warn!("Failed to emit MessageReceived event: {}", e);
        }

        Ok(final_message)
    }

    /// Process a user message with streaming response
    ///
    /// This method streams the response back through Tauri events as it's generated.
    pub async fn respond_streaming(
        &self,
        ctx: &AgentContext,
        user_message: &str,
    ) -> AgentResult<Message> {
        info!(
            "Agent streaming response in session {}",
            ctx.session.id
        );

        // Load conversation history
        let history = ctx.db.list_messages(ctx.session.id)?;

        // Build messages for the LLM
        let mut messages = vec![ChatMessage::system(&self.system_prompt)];

        for msg in &history {
            messages.push(self.message_to_chat_message(msg));
        }

        messages.push(ChatMessage::user(user_message));

        // Get tool definitions
        let tools = self.tool_definitions();

        // Create tool context
        let tool_ctx = ToolContext::new(
            ctx.project_path.clone(),
            ctx.project_id,
            ctx.db.clone(),
            ctx.app_handle.clone(),
            ctx.session.id,
        );

        let mut response_text = String::new();
        let mut all_tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool_call: Option<(String, String, String)> = None; // (id, name, partial_json)
        // Stable message ID for streaming - reused across all chunks so the frontend
        // can update the same message in-place rather than creating duplicates
        let mut streaming_message_id = Uuid::new_v4();

        loop {
            // Emit thinking event
            if let Err(e) = ctx.app_handle.emit(
                "leaf-event",
                &LeafEvent::AgentThinking {
                    session_id: ctx.session.id,
                },
            ) {
                warn!("Failed to emit AgentThinking event: {}", e);
            }

            // Get streaming response
            let mut stream = self
                .provider
                .stream_chat(&messages, Some(&tools))
                .await?;

            let mut stop_reason = StopReason::EndTurn;
            let mut pending_tool_calls: Vec<ToolCallRequest> = Vec::new();

            // Process stream chunks
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;

                match chunk {
                    ChatChunk::ContentBlockStart { content_type, .. } => {
                        if let ContentBlockType::ToolUse { id, name } = content_type {
                            current_tool_call = Some((id, name, String::new()));
                        }
                    }
                    ChatChunk::TextDelta { text, .. } => {
                        response_text.push_str(&text);

                        // Create partial message with stable ID for streaming updates
                        let partial_message = Message {
                            id: streaming_message_id,
                            session_id: ctx.session.id,
                            role: MessageRole::Assistant,
                            content: response_text.clone(),
                            tool_calls: Vec::new(),
                            created_at: Utc::now(),
                        };

                        // Emit partial message (frontend updates in-place via matching ID)
                        if let Err(e) = ctx.app_handle.emit(
                            "leaf-event",
                            &LeafEvent::MessageReceived(partial_message),
                        ) {
                            warn!("Failed to emit partial MessageReceived: {}", e);
                        }
                    }
                    ChatChunk::ToolUseDelta { partial_json, .. } => {
                        if let Some((_, _, ref mut json)) = current_tool_call {
                            json.push_str(&partial_json);
                        }
                    }
                    ChatChunk::ContentBlockStop { .. } => {
                        // If we have a pending tool call, finalize it
                        if let Some((id, name, json)) = current_tool_call.take() {
                            let arguments: serde_json::Value =
                                serde_json::from_str(&json).unwrap_or(serde_json::json!({}));

                            pending_tool_calls.push(ToolCallRequest {
                                id,
                                name,
                                arguments,
                            });
                        }
                    }
                    ChatChunk::MessageStop { stop_reason: reason } => {
                        stop_reason = reason;
                    }
                    ChatChunk::Error { message } => {
                        return Err(AgentError::ProviderError(message));
                    }
                    _ => {}
                }
            }

            // Handle tool calls
            if stop_reason == StopReason::ToolUse && !pending_tool_calls.is_empty() {
                // Add assistant message with tool calls
                let mut assistant_msg = ChatMessage::assistant(&response_text);
                assistant_msg.tool_calls = pending_tool_calls.clone();
                messages.push(assistant_msg);

                // Execute tools
                for tool_call in &pending_tool_calls {
                    info!("Executing tool: {}", tool_call.name);

                    if let Err(e) = ctx.app_handle.emit(
                        "leaf-event",
                        &LeafEvent::AgentToolCall {
                            session_id: ctx.session.id,
                            tool_name: tool_call.name.clone(),
                            arguments: tool_call.arguments.clone(),
                        },
                    ) {
                        warn!("Failed to emit AgentToolCall event: {}", e);
                    }

                    let result = self
                        .tools
                        .execute(&tool_call.name, &tool_ctx, tool_call.arguments.clone())
                        .await;

                    match result {
                        Ok(value) => {
                            messages.push(ChatMessage::tool_result(&tool_call.id, value.clone()));
                            all_tool_calls.push(ToolCall {
                                id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                arguments: tool_call.arguments.clone(),
                                result: Some(value),
                            });
                        }
                        Err(e) => {
                            messages.push(ChatMessage::tool_error(&tool_call.id, e.to_string()));
                            all_tool_calls.push(ToolCall {
                                id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                arguments: tool_call.arguments.clone(),
                                result: Some(serde_json::json!({ "error": e.to_string() })),
                            });
                        }
                    }
                }

                // Reset for next iteration
                response_text.clear();
                streaming_message_id = Uuid::new_v4();
                continue;
            }

            // Done
            break;
        }

        // Create final message with the same stable ID used during streaming
        let final_message = Message {
            id: streaming_message_id,
            session_id: ctx.session.id,
            role: MessageRole::Assistant,
            content: response_text,
            tool_calls: all_tool_calls,
            created_at: Utc::now(),
        };

        // Emit final message
        if let Err(e) = ctx.app_handle.emit(
            "leaf-event",
            &LeafEvent::MessageReceived(final_message.clone()),
        ) {
            warn!("Failed to emit final MessageReceived: {}", e);
        }

        Ok(final_message)
    }

    /// Convert a LEAF Message to a ChatMessage for the LLM
    fn message_to_chat_message(&self, msg: &Message) -> ChatMessage {
        match msg.role {
            MessageRole::User => ChatMessage::user(&msg.content),
            MessageRole::Assistant => {
                let mut chat_msg = ChatMessage::assistant(&msg.content);
                chat_msg.tool_calls = msg
                    .tool_calls
                    .iter()
                    .map(|tc| ToolCallRequest {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    })
                    .collect();
                chat_msg
            }
            MessageRole::System => ChatMessage::system(&msg.content),
            MessageRole::Tool => {
                // Tool messages are converted to tool results
                if let Some(tc) = msg.tool_calls.first() {
                    if let Some(ref result) = tc.result {
                        ChatMessage::tool_result(&tc.id, result.clone())
                    } else {
                        ChatMessage::user(&msg.content)
                    }
                } else {
                    ChatMessage::user(&msg.content)
                }
            }
        }
    }
}

/// Builder for creating an Agent with specific configuration
pub struct AgentBuilder {
    config: ProviderConfig,
    tools: Option<ToolRegistry>,
    system_prompt: Option<String>,
}

impl AgentBuilder {
    /// Create a new agent builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ProviderConfig::default(),
            tools: None,
            system_prompt: None,
        }
    }

    /// Set the provider configuration
    pub fn with_config(mut self, config: ProviderConfig) -> Self {
        self.config = config;
        self
    }

    /// Set a custom tool registry
    pub fn with_tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set a custom system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Build the agent
    pub fn build(self) -> AgentResult<Agent> {
        let provider = create_provider(self.config)?;
        let tools = self.tools.unwrap_or_else(ToolRegistry::with_defaults);
        let system_prompt = self.system_prompt.unwrap_or_else(prompts::system_prompt);

        Ok(Agent {
            provider,
            tools,
            system_prompt,
        })
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{ChatRole, ProviderType};

    #[test]
    fn test_agent_builder() {
        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            api_key: Some("test-key".to_string()),
            model: "claude-sonnet-4-20250514".to_string(),
            ..Default::default()
        };

        let result = AgentBuilder::new().with_config(config).build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_message_to_chat_message() {
        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let agent = Agent::new(config).unwrap();
        let session_id = Uuid::new_v4();

        let user_msg = Message::new(session_id, MessageRole::User, "Hello");
        let chat_msg = agent.message_to_chat_message(&user_msg);
        assert_eq!(chat_msg.role, ChatRole::User);
        assert_eq!(chat_msg.content, "Hello");

        let assistant_msg = Message::new(session_id, MessageRole::Assistant, "Hi there");
        let chat_msg = agent.message_to_chat_message(&assistant_msg);
        assert_eq!(chat_msg.role, ChatRole::Assistant);
    }
}
