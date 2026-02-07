//! Event bus for LEAF internal events

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{
    Artifact, Card, ChatSession, Event, ExecutionStatus, EventStatus, Message, Project,
    Stack, StackExecution, CardExecution,
};

/// Events emitted by LEAF for UI updates and inter-component communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LeafEvent {
    // Project events
    ProjectOpened(Project),
    ProjectClosed { project_id: Uuid },
    ProjectUpdated(Project),

    // Stack events
    StackCreated(Stack),
    StackUpdated(Stack),
    StackDeleted { stack_id: Uuid },
    StackEnabled { stack_id: Uuid },
    StackDisabled { stack_id: Uuid },

    // Card events
    CardCreated(Card),
    CardUpdated(Card),
    CardDeleted { card_id: Uuid, stack_id: Uuid },
    CardReordered { stack_id: Uuid },

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
        matched_stacks: Vec<Uuid>,
    },
    EventCompleted {
        event_id: Uuid,
        status: EventStatus,
    },

    // Stack execution events
    StackExecutionStarted(StackExecution),
    StackExecutionProgress {
        stack_execution_id: Uuid,
        completed_cards: i32,
        card_count: i32,
    },
    StackExecutionCompleted {
        stack_execution_id: Uuid,
        status: ExecutionStatus,
    },

    // Card execution events
    CardExecutionStarted(CardExecution),
    CardExecutionCompleted {
        card_execution_id: Uuid,
        status: ExecutionStatus,
        exit_code: Option<i32>,
    },

    // Artifact events
    ArtifactCreated(Artifact),
    ArtifactModified { artifact_id: Uuid },
    ArtifactDeleted { artifact_id: Uuid },

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
        let event = LeafEvent::StackEnabled {
            stack_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("stack_enabled"));
    }

    #[test]
    fn test_leaf_event_with_payload() {
        let project = Project::new("Test", "/path");
        let event = LeafEvent::ProjectOpened(project);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("project_opened"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_leaf_event_card_deleted_has_stack_id() {
        let event = LeafEvent::CardDeleted {
            card_id: Uuid::new_v4(),
            stack_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("card_deleted"));
        assert!(json.contains("stack_id"));
    }

    #[test]
    fn test_leaf_event_stack_execution() {
        let stack_id = Uuid::new_v4();
        let exec = StackExecution::new(stack_id, None, 2);
        let event = LeafEvent::StackExecutionStarted(exec);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("stack_execution_started"));
    }
}
