//! Card creation tools for the LEAF agent
//!
//! These tools allow the agent to propose and create automation cards.

use async_trait::async_trait;
use leaf_core::{Card, LeafEvent, ProgramConfig, TriggerConfig};
use leaf_db::CardQueries;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Emitter;
use tracing::{debug, info};

use super::{Tool, ToolContext};
use crate::codegen::generate_card_program;
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

#[derive(Debug, Deserialize)]
struct ProposeCardArgs {
    name: String,
    description: String,
    trigger: TriggerProposal,
    /// The TypeScript code for the card
    code: String,
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
        let args: ProposeCardArgs = serde_json::from_value(args)?;

        debug!("Proposing card: {}", args.name);

        // Generate the full program code with boilerplate
        let full_code = generate_card_program(&args.code)?;

        let proposal = CardProposal {
            name: args.name,
            description: args.description,
            trigger: args.trigger,
            code_preview: full_code,
        };

        Ok(serde_json::json!({
            "status": "proposed",
            "message": "Card proposed for user review. Wait for user confirmation before creating.",
            "proposal": proposal
        }))
    }
}

/// Tool for creating a card immediately
pub struct CreateCardNowTool;

#[derive(Debug, Deserialize)]
struct CreateCardArgs {
    name: String,
    description: String,
    trigger: TriggerProposal,
    /// The TypeScript code for the card
    code: String,
}

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
        let args: CreateCardArgs = serde_json::from_value(args)?;

        info!("Creating card: {}", args.name);

        // Generate the full program code with boilerplate
        let full_code = generate_card_program(&args.code)?;

        // Create the card slug from the name
        let slug = slugify(&args.name);

        // Create the program directory
        let program_dir = ctx.programs_dir().join(&slug);
        tokio::fs::create_dir_all(&program_dir)
            .await
            .map_err(|e| AgentError::IoError(e))?;

        // Write the TypeScript code
        let entrypoint = "main.ts";
        let program_path = program_dir.join(entrypoint);
        tokio::fs::write(&program_path, &full_code)
            .await
            .map_err(|e| AgentError::IoError(e))?;

        debug!("Wrote program to: {}", program_path.display());

        // Create the card in the database
        let mut card = Card::new(ctx.project_id, &args.name, &args.description);
        card.trigger = args.trigger.into();
        card.program = ProgramConfig {
            language: "typescript".to_string(),
            entrypoint: entrypoint.to_string(),
            dependencies: Vec::new(),
            timeout_secs: 300,
            max_retries: 3,
        };
        card.session_id = Some(ctx.session_id);

        // Save to database
        ctx.db.create_card(&card)?;

        info!("Card created: {} ({})", card.name, card.id);

        // Emit card created event
        if let Err(e) = ctx.app_handle.emit("leaf-event", &LeafEvent::CardCreated(card.clone())) {
            tracing::warn!("Failed to emit CardCreated event: {}", e);
        }

        Ok(serde_json::json!({
            "status": "created",
            "card_id": card.id.to_string(),
            "name": card.name,
            "program_path": program_path.to_string_lossy(),
            "message": format!("Card '{}' created successfully!", card.name)
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
