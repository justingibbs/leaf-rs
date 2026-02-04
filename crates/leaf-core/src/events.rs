//! Event bus for LEAF internal events

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{
    Card, ChatSession, Event, Execution, ExecutionStatus, EventStatus, Message, Project,
};

/// Events emitted by LEAF for UI updates and inter-component communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LeafEvent {
    // Project events
    ProjectOpened(Project),
    ProjectClosed { project_id: Uuid },
    ProjectUpdated(Project),

    // Card events
    CardCreated(Card),
    CardUpdated(Card),
    CardDeleted { card_id: Uuid },
    CardEnabled { card_id: Uuid },
    CardDisabled { card_id: Uuid },

    // File watcher events
    FileDetected {
        project_id: Uuid,
        path: String,
        event_type: String,
    },
    WatcherStarted {
        project_id: Uuid,
        paths: Vec<String>,
    },
    WatcherStopped {
        project_id: Uuid,
    },
    WatcherError {
        project_id: Uuid,
        error: String,
    },

    // Event processing
    EventCreated(Event),
    EventProcessing {
        event_id: Uuid,
        matched_cards: Vec<Uuid>,
    },
    EventCompleted {
        event_id: Uuid,
        status: EventStatus,
    },

    // Execution events
    ExecutionStarted(Execution),
    ExecutionProgress {
        execution_id: Uuid,
        stdout: String,
        stderr: String,
    },
    ExecutionCompleted {
        execution_id: Uuid,
        status: ExecutionStatus,
        exit_code: Option<i32>,
    },

    // Chat events
    SessionCreated(ChatSession),
    SessionUpdated(ChatSession),
    MessageReceived(Message),
    AgentThinking {
        session_id: Uuid,
    },
    AgentToolCall {
        session_id: Uuid,
        tool_name: String,
        arguments: serde_json::Value,
    },

    // MCP events
    McpServerConnected {
        server_id: Uuid,
        server_name: String,
        tool_count: usize,
    },
    McpServerDisconnected {
        server_id: Uuid,
        server_name: String,
    },
    McpServerError {
        server_id: Uuid,
        server_name: String,
        error: String,
    },
    McpToolCalled {
        session_id: Uuid,
        server_name: String,
        tool_name: String,
    },
    McpToolResult {
        session_id: Uuid,
        server_name: String,
        tool_name: String,
        success: bool,
    },

    // System events
    Error {
        context: String,
        message: String,
    },
    Warning {
        context: String,
        message: String,
    },
}

/// Trait for components that can emit events
pub trait EventBus: Send + Sync {
    /// Emit an event to all listeners
    fn emit(&self, event: LeafEvent);

    /// Subscribe to events (implementation-specific)
    fn subscribe(&self) -> Box<dyn EventSubscription>;
}

/// Trait for event subscriptions
pub trait EventSubscription: Send {
    /// Receive the next event (blocking)
    fn recv(&self) -> Option<LeafEvent>;

    /// Try to receive an event (non-blocking)
    fn try_recv(&self) -> Option<LeafEvent>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_event_serialization() {
        let event = LeafEvent::CardEnabled {
            card_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("card_enabled"));
    }

    #[test]
    fn test_leaf_event_with_payload() {
        let project = Project::new("Test", "/path");
        let event = LeafEvent::ProjectOpened(project);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("project_opened"));
        assert!(json.contains("Test"));
    }
}
