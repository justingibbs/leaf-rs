//! Core domain types for LEAF

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A LEAF project - a folder containing automations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub last_opened_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            path: path.into(),
            created_at: now,
            last_opened_at: now,
        }
    }
}

/// Trigger configuration - what causes a card to run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerConfig {
    /// Trigger when a file is created in watched folder
    FileCreated {
        watch_path: String,
        #[serde(default)]
        patterns: Vec<String>,
    },
    /// Trigger when a file is modified
    FileModified {
        watch_path: String,
        #[serde(default)]
        patterns: Vec<String>,
    },
    /// Trigger on a schedule (cron expression)
    Schedule { cron: String },
    /// Manual trigger only
    #[default]
    Manual,
}

impl TriggerConfig {
    pub fn is_manual(&self) -> bool {
        matches!(self, Self::Manual)
    }
}

/// Program configuration - how to run the card's code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// Programming language (currently only "typescript")
    pub language: String,
    /// Entry point file name
    pub entrypoint: String,
    /// NPM dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u32 {
    300
}

fn default_max_retries() -> u32 {
    3
}

impl Default for ProgramConfig {
    fn default() -> Self {
        Self {
            language: "typescript".to_string(),
            entrypoint: "main.ts".to_string(),
            dependencies: Vec::new(),
            timeout_secs: default_timeout(),
            max_retries: default_max_retries(),
        }
    }
}

/// A Card - an automation unit with trigger and program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger: TriggerConfig,
    pub program: ProgramConfig,
    pub enabled: bool,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    pub fn new(
        project_id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            name: name.into(),
            description: description.into(),
            trigger: TriggerConfig::default(),
            program: ProgramConfig::default(),
            enabled: true,
            session_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Event type - what triggered an event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    FileCreated,
    FileModified,
    Schedule,
    Manual,
}

/// Event status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    #[default]
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Event payload - data associated with an event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    File {
        path: String,
        #[serde(default)]
        size: Option<u64>,
        #[serde(default)]
        mime_type: Option<String>,
    },
    Schedule {
        scheduled_time: DateTime<Utc>,
    },
    Manual {
        #[serde(default)]
        input: Option<String>,
    },
}

/// An Event - something that happened that may trigger cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub project_id: Uuid,
    pub event_type: EventType,
    pub payload: EventPayload,
    pub status: EventStatus,
    pub matched_cards: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl Event {
    pub fn new(project_id: Uuid, event_type: EventType, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            event_type,
            payload,
            status: EventStatus::default(),
            matched_cards: Vec::new(),
            created_at: Utc::now(),
            processed_at: None,
        }
    }
}

/// Execution status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
    Cancelled,
}

/// An Execution - a single run of a card's program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: Uuid,
    pub card_id: Uuid,
    pub event_id: Option<Uuid>,
    pub status: ExecutionStatus,
    pub attempt: u32,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

impl Execution {
    pub fn new(card_id: Uuid, event_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_id,
            event_id,
            status: ExecutionStatus::default(),
            attempt: 1,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
        }
    }
}

/// Chat session status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Active,
    Completed,
    Archived,
}

/// A ChatSession - a conversation with the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChatSession {
    pub fn new(project_id: Uuid, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            title: title.into(),
            status: SessionStatus::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Message role in a chat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool call in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

/// A Message in a chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(session_id: Uuid, role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role,
            content: content.into(),
            tool_calls: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl McpServer {
    pub fn new(
        project_id: Uuid,
        name: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("Test Project", "/path/to/project");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.path, "/path/to/project");
    }

    #[test]
    fn test_card_new() {
        let project_id = Uuid::new_v4();
        let card = Card::new(project_id, "Test Card", "A test card");
        assert_eq!(card.name, "Test Card");
        assert_eq!(card.project_id, project_id);
        assert!(card.enabled);
    }

    #[test]
    fn test_trigger_config_serialization() {
        let trigger = TriggerConfig::FileCreated {
            watch_path: "/inbox".to_string(),
            patterns: vec!["*.pdf".to_string()],
        };
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("file_created"));
    }

    #[test]
    fn test_event_payload_serialization() {
        let payload = EventPayload::File {
            path: "/test.pdf".to_string(),
            size: Some(1024),
            mime_type: Some("application/pdf".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: EventPayload = serde_json::from_str(&json).unwrap();
        match parsed {
            EventPayload::File { path, .. } => assert_eq!(path, "/test.pdf"),
            _ => panic!("Wrong variant"),
        }
    }
}
