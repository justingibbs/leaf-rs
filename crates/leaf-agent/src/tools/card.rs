//! Card creation tools for the LEAF agent
//!
//! These tools allow the agent to propose and create automation cards.
//! - `propose_card`: Returns a preview JSON without persisting (informational only)
//! - `create_card_now`: Creates a Stack + Card, writes program files, persists to DB

use async_trait::async_trait;
use leaf_core::{Card, LeafEvent, ProgramConfig, Stack, TriggerConfig};
use leaf_db::{CardQueries, StackQueries};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Emitter;
use tracing::{debug, info, warn};

use super::{Tool, ToolContext};
use crate::error::{AgentError, AgentResult};

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

/// Parse trigger from JSON args
fn parse_trigger(trigger_value: &Value) -> AgentResult<TriggerProposal> {
    serde_json::from_value(trigger_value.clone()).map_err(|e| {
        AgentError::InvalidToolCall(format!("Invalid trigger: {}", e))
    })
}

/// Extract a required string field from args
fn get_required_str<'a>(args: &'a Value, field: &str) -> AgentResult<&'a str> {
    args.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::InvalidToolCall(format!("{} is required", field)))
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

    async fn execute(&self, _ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let name = get_required_str(&args, "name")?;
        let description = get_required_str(&args, "description")?;
        let code = get_required_str(&args, "code")?;
        let trigger_value = args
            .get("trigger")
            .ok_or_else(|| AgentError::InvalidToolCall("trigger is required".to_string()))?;

        let trigger_proposal = parse_trigger(trigger_value)?;
        let trigger_config: TriggerConfig = trigger_proposal.clone().into();

        debug!("propose_card: name={}, trigger={:?}", name, trigger_config);

        // Build preview — no DB writes, no events
        Ok(serde_json::json!({
            "status": "proposed",
            "stack": {
                "name": name,
                "description": description,
                "trigger": trigger_config,
            },
            "card": {
                "name": name,
                "description": description,
                "program_path": format!("programs/{}", slugify(name)),
            },
            "code_preview": code,
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

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let name = get_required_str(&args, "name")?.to_string();
        let description = get_required_str(&args, "description")?.to_string();
        let code = get_required_str(&args, "code")?.to_string();
        let trigger_value = args
            .get("trigger")
            .ok_or_else(|| AgentError::InvalidToolCall("trigger is required".to_string()))?;

        let trigger_proposal = parse_trigger(trigger_value)?;
        let trigger_config: TriggerConfig = trigger_proposal.into();

        // 1. Create Stack
        let mut stack = Stack::new(ctx.project_id, &name, &description);
        stack.trigger = trigger_config;
        stack.source_session_id = Some(ctx.session_id);

        // 2. Create Card
        let slug = slugify(&name);
        let program_path = format!("programs/{}", slug);

        let mut card = Card::new(stack.id, &name, &description);
        card.program_path = program_path.clone();
        card.program = ProgramConfig {
            language: "typescript".to_string(),
            entrypoint: "main.ts".to_string(),
            dependencies: Vec::new(),
            timeout_secs: 300,
            max_retries: 3,
        };

        // 3. Write program files to disk
        let program_dir = ctx.programs_dir().join(&slug);
        std::fs::create_dir_all(&program_dir)?;

        let main_ts_path = program_dir.join("main.ts");
        std::fs::write(&main_ts_path, &code)?;

        info!(
            "Wrote program to {}",
            main_ts_path.display()
        );

        // 4. Persist to DB
        ctx.db.create_stack(&stack)?;
        ctx.db.create_card(&card)?;

        info!(
            "Created stack {} and card {} for '{}'",
            stack.id, card.id, name
        );

        // 5. Emit events
        if let Err(e) = ctx
            .app_handle
            .emit("leaf-event", &LeafEvent::StackCreated(stack.clone()))
        {
            warn!("Failed to emit StackCreated event: {}", e);
        }
        if let Err(e) = ctx
            .app_handle
            .emit("leaf-event", &LeafEvent::CardCreated(card.clone()))
        {
            warn!("Failed to emit CardCreated event: {}", e);
        }

        // 6. Return success
        Ok(serde_json::json!({
            "status": "created",
            "stack_id": stack.id.to_string(),
            "card_id": card.id.to_string(),
            "program_path": program_path,
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
