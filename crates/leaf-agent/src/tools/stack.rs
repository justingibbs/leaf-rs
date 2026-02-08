//! Stack creation tools for the LEAF agent
//!
//! These tools allow the agent to propose and create automation stacks with cards.
//! - `propose_stack`: Returns a preview JSON without persisting (for user review)
//! - `create_stack_now`: Creates a Stack + Cards, writes program files, persists to DB
//! - `add_card`: Adds a card to an existing stack

use async_trait::async_trait;
use leaf_core::{Card, LeafEvent, ProgramConfig, Stack, TriggerConfig};
use leaf_db::{CardQueries, StackQueries};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Emitter;
use tracing::{debug, info, warn};

use super::{Tool, ToolContext};
use crate::error::{AgentError, AgentResult};

/// Tool for proposing a stack (preview before creation)
pub struct ProposeStackTool;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CardProposalInput {
    name: String,
    #[serde(default)]
    description: Option<String>,
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerProposal {
    FileCreated {
        watch_path: String,
        #[serde(default)]
        patterns: Vec<String>,
    },
    FileModified {
        watch_path: String,
        #[serde(default)]
        patterns: Vec<String>,
    },
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

/// Convert a name to a slug for the program directory
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

/// Parse the cards array from args
fn parse_cards(args: &Value) -> AgentResult<Vec<CardProposalInput>> {
    let cards_value = args
        .get("cards")
        .ok_or_else(|| AgentError::InvalidToolCall("cards is required".to_string()))?;

    let cards: Vec<CardProposalInput> = serde_json::from_value(cards_value.clone())
        .map_err(|e| AgentError::InvalidToolCall(format!("Invalid cards array: {}", e)))?;

    if cards.is_empty() {
        return Err(AgentError::InvalidToolCall(
            "cards must contain at least one card".to_string(),
        ));
    }

    Ok(cards)
}

/// Create a Card struct from proposal input
fn build_card(stack_id: uuid::Uuid, input: &CardProposalInput, position: i32) -> Card {
    let slug = slugify(&input.name);
    let program_path = format!("programs/{}", slug);

    let mut card = Card::new(
        stack_id,
        &input.name,
        input.description.as_deref().unwrap_or(""),
    );
    card.program_path = program_path;
    card.position = position;
    card.program = ProgramConfig {
        language: "typescript".to_string(),
        entrypoint: "main.ts".to_string(),
        dependencies: Vec::new(),
        timeout_secs: 300,
        max_retries: 3,
    };

    card
}

#[async_trait]
impl Tool for ProposeStackTool {
    fn name(&self) -> &str {
        "propose_stack"
    }

    fn description(&self) -> &str {
        "Propose a new automation stack with one or more cards for the user to review before creation. Use this to show the user what will be created and get their confirmation."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Propose a new stack with cards for user review",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Stack name (e.g., 'CSV Pipeline')"
                },
                "description": {
                    "type": "string",
                    "description": "What this workflow does"
                },
                "trigger": {
                    "type": "object",
                    "description": "When the stack should run",
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
                "cards": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Card step name" },
                            "description": { "type": "string", "description": "What this step does" },
                            "code": { "type": "string", "description": "TypeScript code for this step" }
                        },
                        "required": ["name", "code"]
                    },
                    "minItems": 1,
                    "description": "Ordered list of card steps in the pipeline"
                }
            },
            "required": ["name", "trigger", "cards"]
        })
    }

    async fn execute(&self, _ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let name = get_required_str(&args, "name")?;
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let trigger_value = args
            .get("trigger")
            .ok_or_else(|| AgentError::InvalidToolCall("trigger is required".to_string()))?;
        let trigger_proposal = parse_trigger(trigger_value)?;
        let trigger_config: TriggerConfig = trigger_proposal.clone().into();
        let cards = parse_cards(&args)?;

        debug!(
            "propose_stack: name={}, trigger={:?}, cards={}",
            name,
            trigger_config,
            cards.len()
        );

        let card_previews: Vec<Value> = cards
            .iter()
            .enumerate()
            .map(|(i, c)| {
                serde_json::json!({
                    "position": i,
                    "name": c.name,
                    "description": c.description,
                    "program_path": format!("programs/{}", slugify(&c.name)),
                    "code_preview": c.code,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "status": "proposed",
            "stack": {
                "name": name,
                "description": description,
                "trigger": trigger_config,
            },
            "cards": card_previews,
        }))
    }
}

/// Tool for creating a stack immediately
pub struct CreateStackNowTool;

#[async_trait]
impl Tool for CreateStackNowTool {
    fn name(&self) -> &str {
        "create_stack_now"
    }

    fn description(&self) -> &str {
        "Create a new automation stack with cards immediately. Only use this after the user has confirmed they want the stack created, or if they explicitly asked to create without preview."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Create a new stack with cards immediately",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Stack name (e.g., 'CSV Pipeline')"
                },
                "description": {
                    "type": "string",
                    "description": "What this workflow does"
                },
                "trigger": {
                    "type": "object",
                    "description": "When the stack should run",
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
                "cards": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Card step name" },
                            "description": { "type": "string", "description": "What this step does" },
                            "code": { "type": "string", "description": "TypeScript code for this step" }
                        },
                        "required": ["name", "code"]
                    },
                    "minItems": 1,
                    "description": "Ordered list of card steps in the pipeline"
                }
            },
            "required": ["name", "trigger", "cards"]
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let name = get_required_str(&args, "name")?.to_string();
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let trigger_value = args
            .get("trigger")
            .ok_or_else(|| AgentError::InvalidToolCall("trigger is required".to_string()))?;
        let trigger_proposal = parse_trigger(trigger_value)?;
        let trigger_config: TriggerConfig = trigger_proposal.into();
        let card_inputs = parse_cards(&args)?;

        // 1. Create Stack
        let mut stack = Stack::new(ctx.project_id, &name, &description);
        stack.trigger = trigger_config;
        stack.source_session_id = Some(ctx.session_id);

        // 2. Create Cards and write program files
        let mut created_cards = Vec::new();
        for (i, input) in card_inputs.iter().enumerate() {
            let card = build_card(stack.id, input, i as i32);

            // Write program file
            let slug = slugify(&input.name);
            let program_dir = ctx.programs_dir().join(&slug);
            std::fs::create_dir_all(&program_dir)?;
            let main_ts_path = program_dir.join("main.ts");
            std::fs::write(&main_ts_path, &input.code)?;

            info!(
                "Wrote program to {}",
                main_ts_path.display()
            );

            created_cards.push(card);
        }

        // 3. Persist to DB
        ctx.db.create_stack(&stack)?;
        for card in &created_cards {
            ctx.db.create_card(card)?;
        }

        info!(
            "Created stack {} with {} cards for '{}'",
            stack.id,
            created_cards.len(),
            name
        );

        // 4. Emit events
        if let Err(e) = ctx
            .app_handle
            .emit("leaf-event", &LeafEvent::StackCreated(stack.clone()))
        {
            warn!("Failed to emit StackCreated event: {}", e);
        }
        for card in &created_cards {
            if let Err(e) = ctx
                .app_handle
                .emit("leaf-event", &LeafEvent::CardCreated(card.clone()))
            {
                warn!("Failed to emit CardCreated event: {}", e);
            }
        }

        // 5. Return success
        let card_ids: Vec<Value> = created_cards
            .iter()
            .map(|c| {
                serde_json::json!({
                    "card_id": c.id.to_string(),
                    "name": c.name,
                    "position": c.position,
                    "program_path": c.program_path,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "status": "created",
            "stack_id": stack.id.to_string(),
            "cards": card_ids,
        }))
    }
}

/// Tool for adding a card to an existing stack
pub struct AddCardTool;

#[async_trait]
impl Tool for AddCardTool {
    fn name(&self) -> &str {
        "add_card"
    }

    fn description(&self) -> &str {
        "Add a new card step to an existing stack. The card is appended to the end of the pipeline by default, or inserted at the specified position."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Add a card to an existing stack",
            "properties": {
                "stack_id": {
                    "type": "string",
                    "description": "The stack to add this card to"
                },
                "name": {
                    "type": "string",
                    "description": "Card step name"
                },
                "description": {
                    "type": "string",
                    "description": "What this step does"
                },
                "code": {
                    "type": "string",
                    "description": "TypeScript code for this step"
                },
                "position": {
                    "type": "integer",
                    "description": "Position in the pipeline (appended to end if omitted)"
                }
            },
            "required": ["stack_id", "name", "code"]
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let stack_id_str = get_required_str(&args, "stack_id")?;
        let stack_id: uuid::Uuid = stack_id_str
            .parse()
            .map_err(|e| AgentError::InvalidToolCall(format!("Invalid stack_id: {}", e)))?;
        let name = get_required_str(&args, "name")?.to_string();
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let code = get_required_str(&args, "code")?.to_string();

        // Verify stack exists
        let stack = ctx
            .db
            .get_stack(stack_id)?
            .ok_or_else(|| AgentError::ToolError(format!("Stack {} not found", stack_id)))?;

        // Determine position
        let existing_cards = ctx.db.list_cards_for_stack(stack_id)?;
        let position = args
            .get("position")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(existing_cards.len() as i32);

        // Build card
        let input = CardProposalInput {
            name: name.clone(),
            description: Some(description),
            code: code.clone(),
        };
        let card = build_card(stack.id, &input, position);

        // Write program file
        let slug = slugify(&name);
        let program_dir = ctx.programs_dir().join(&slug);
        std::fs::create_dir_all(&program_dir)?;
        let main_ts_path = program_dir.join("main.ts");
        std::fs::write(&main_ts_path, &code)?;

        info!(
            "Wrote program to {}",
            main_ts_path.display()
        );

        // Persist to DB
        ctx.db.create_card(&card)?;

        info!(
            "Added card {} to stack {} at position {}",
            card.id, stack.id, position
        );

        // Emit event
        if let Err(e) = ctx
            .app_handle
            .emit("leaf-event", &LeafEvent::CardCreated(card.clone()))
        {
            warn!("Failed to emit CardCreated event: {}", e);
        }

        Ok(serde_json::json!({
            "status": "created",
            "card_id": card.id.to_string(),
            "stack_id": stack.id.to_string(),
            "name": card.name,
            "position": card.position,
            "program_path": card.program_path,
        }))
    }
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

    #[test]
    fn test_parse_cards_valid() {
        let args = serde_json::json!({
            "cards": [
                {"name": "Step 1", "code": "console.log('step 1');"},
                {"name": "Step 2", "description": "Second step", "code": "console.log('step 2');"}
            ]
        });
        let cards = parse_cards(&args).unwrap();
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].name, "Step 1");
        assert!(cards[0].description.is_none());
        assert_eq!(cards[1].description.as_deref(), Some("Second step"));
    }

    #[test]
    fn test_parse_cards_empty_rejected() {
        let args = serde_json::json!({ "cards": [] });
        assert!(parse_cards(&args).is_err());
    }

    #[test]
    fn test_parse_cards_missing_rejected() {
        let args = serde_json::json!({ "name": "test" });
        assert!(parse_cards(&args).is_err());
    }

    #[test]
    fn test_build_card() {
        let stack_id = uuid::Uuid::new_v4();
        let input = CardProposalInput {
            name: "Parse CSV".to_string(),
            description: Some("Parses the CSV file".to_string()),
            code: "console.log('hello');".to_string(),
        };
        let card = build_card(stack_id, &input, 0);
        assert_eq!(card.stack_id, stack_id);
        assert_eq!(card.name, "Parse CSV");
        assert_eq!(card.description, "Parses the CSV file");
        assert_eq!(card.program_path, "programs/parse-csv");
        assert_eq!(card.position, 0);
        assert_eq!(card.program.language, "typescript");
        assert_eq!(card.program.entrypoint, "main.ts");
    }

    #[test]
    fn test_build_card_no_description() {
        let stack_id = uuid::Uuid::new_v4();
        let input = CardProposalInput {
            name: "Step".to_string(),
            description: None,
            code: "".to_string(),
        };
        let card = build_card(stack_id, &input, 2);
        assert_eq!(card.description, "");
        assert_eq!(card.position, 2);
    }
}
