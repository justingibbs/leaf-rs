//! Card creation tools for the LEAF agent
//!
//! These tools allow the agent to propose and create automation cards.
//! Phase A: Execute methods are stubbed to return "temporarily disabled" messages.
//! The struct definitions and Tool trait implementations are preserved so the tool registry compiles.

use async_trait::async_trait;
use leaf_core::TriggerConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use super::{Tool, ToolContext};
use crate::error::AgentResult;

/// Tool for proposing a card (preview before creation)
pub struct ProposeCardTool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardProposal {
    /// Name of the card
    pub name: String,
    /// Description of what the card does
    pub description: String,
    /// Trigger configuration
    pub trigger: TriggerProposal,
    /// The TypeScript code that will be generated
    pub code_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerProposal {
    /// Trigger when a file is created
    FileCreated {
        /// Folder to watch (relative to project root)
        watch_path: String,
        /// File patterns to match (e.g., ["*.csv", "*.xlsx"])
        #[serde(default)]
        patterns: Vec<String>,
    },
    /// Trigger when a file is modified
    FileModified {
        watch_path: String,
        #[serde(default)]
        patterns: Vec<String>,
    },
    /// Manual trigger only
    Manual,
}

impl From<TriggerProposal> for TriggerConfig {
    fn from(proposal: TriggerProposal) -> Self {
        match proposal {
            TriggerProposal::FileCreated {
                watch_path,
                patterns,
            } => TriggerConfig::FileCreated {
                watch_path,
                patterns,
            },
            TriggerProposal::FileModified {
                watch_path,
                patterns,
            } => TriggerConfig::FileModified {
                watch_path,
                patterns,
            },
            TriggerProposal::Manual => TriggerConfig::Manual,
        }
    }
}

#[async_trait]
impl Tool for ProposeCardTool {
    fn name(&self) -> &str {
        "propose_card"
    }

    fn description(&self) -> &str {
        "Propose a new automation card for the user to review before creation. Use this to show the user what card will be created and get their confirmation."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Propose a new card for user review",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the card (short, descriptive)"
                },
                "description": {
                    "type": "string",
                    "description": "Description of what the card does"
                },
                "trigger": {
                    "type": "object",
                    "description": "When the card should run",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["file_created", "file_modified", "manual"],
                            "description": "Type of trigger"
                        },
                        "watch_path": {
                            "type": "string",
                            "description": "Folder to watch (for file triggers)"
                        },
                        "patterns": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "File patterns to match (e.g., ['*.csv'])"
                        }
                    },
                    "required": ["type"]
                },
                "code": {
                    "type": "string",
                    "description": "The TypeScript code for the card program"
                }
            },
            "required": ["name", "description", "trigger", "code"]
        })
    }

    async fn execute(&self, _ctx: &ToolContext, _args: Value) -> AgentResult<Value> {
        // TODO: Phase B - reimplement with Stack creation
        debug!("propose_card: temporarily disabled during Stack migration");
        Ok(serde_json::json!({
            "status": "error",
            "message": "Card proposal temporarily disabled during Stack & Cards migration. This will be re-enabled in a future update."
        }))
    }
}

/// Tool for creating a card immediately
pub struct CreateCardNowTool;

#[async_trait]
impl Tool for CreateCardNowTool {
    fn name(&self) -> &str {
        "create_card_now"
    }

    fn description(&self) -> &str {
        "Create a new automation card immediately. Only use this after the user has confirmed they want the card created, or if they explicitly asked to create without preview."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Create a new card immediately",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the card (short, descriptive)"
                },
                "description": {
                    "type": "string",
                    "description": "Description of what the card does"
                },
                "trigger": {
                    "type": "object",
                    "description": "When the card should run",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["file_created", "file_modified", "manual"],
                            "description": "Type of trigger"
                        },
                        "watch_path": {
                            "type": "string",
                            "description": "Folder to watch (for file triggers)"
                        },
                        "patterns": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "File patterns to match (e.g., ['*.csv'])"
                        }
                    },
                    "required": ["type"]
                },
                "code": {
                    "type": "string",
                    "description": "The TypeScript code for the card program"
                }
            },
            "required": ["name", "description", "trigger", "code"]
        })
    }

    async fn execute(&self, _ctx: &ToolContext, _args: Value) -> AgentResult<Value> {
        // TODO: Phase B - reimplement with Stack + Card creation
        debug!("create_card_now: temporarily disabled during Stack migration");
        Ok(serde_json::json!({
            "status": "error",
            "message": "Card creation temporarily disabled during Stack & Cards migration. This will be re-enabled in a future update."
        }))
    }
}

/// Convert a card name to a slug for the program directory
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("CSV Analyzer"), "csv-analyzer");
        assert_eq!(slugify("My Card Name"), "my-card-name");
        assert_eq!(slugify("Test123"), "test123");
        assert_eq!(slugify("  Spaced  Out  "), "spaced-out");
    }

    #[test]
    fn test_trigger_proposal_conversion() {
        let proposal = TriggerProposal::FileCreated {
            watch_path: "inbox".to_string(),
            patterns: vec!["*.csv".to_string()],
        };

        let config: TriggerConfig = proposal.into();

        match config {
            TriggerConfig::FileCreated {
                watch_path,
                patterns,
            } => {
                assert_eq!(watch_path, "inbox");
                assert_eq!(patterns, vec!["*.csv".to_string()]);
            }
            _ => panic!("Wrong trigger type"),
        }
    }
}
