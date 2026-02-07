//! Stack execution pipeline
//!
//! Orchestrates running all cards in a stack sequentially via the Deno executor.
//! Creates StackExecution and CardExecution records and emits progress events.

use chrono::Utc;
use leaf_core::{CardExecution, Event, ExecutionStatus, LeafEvent, StackExecution};
use leaf_db::{CardExecutionQueries, CardQueries, Database, StackExecutionQueries};
use leaf_executor::Executor;
use std::path::Path;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Run a stack's execution pipeline.
///
/// Lists enabled cards for the stack (ordered by position), creates execution
/// records, runs each card's program via Deno, and emits progress events.
///
/// Returns the completed StackExecution.
pub async fn run_stack(
    stack_id: Uuid,
    event_id: Option<Uuid>,
    event: Option<&Event>,
    db: &Database,
    executor: &Executor,
    app_handle: &AppHandle,
    project_path: &Path,
) -> Result<StackExecution, String> {
    let start = Instant::now();

    // List enabled cards for this stack, ordered by position
    let cards = db
        .list_enabled_cards_for_stack(stack_id)
        .map_err(|e| format!("Failed to list cards: {}", e))?;

    if cards.is_empty() {
        return Err("Stack has no enabled cards".to_string());
    }

    // Create StackExecution record (status=Running)
    let mut stack_exec = StackExecution::new(stack_id, event_id, cards.len() as i32);
    stack_exec.status = ExecutionStatus::Running;
    db.create_stack_execution(&stack_exec)
        .map_err(|e| format!("Failed to create stack execution: {}", e))?;

    info!(
        "Starting stack execution {} with {} cards",
        stack_exec.id,
        cards.len()
    );

    // Emit StackExecutionStarted
    emit(app_handle, LeafEvent::StackExecutionStarted(stack_exec.clone()));

    // Run each card in order
    let mut pipeline_failed = false;

    for card in &cards {
        // Create CardExecution record (status=Running)
        let mut card_exec = CardExecution::new(stack_exec.id, card.id, card.position);
        card_exec.status = ExecutionStatus::Running;
        db.create_card_execution(&card_exec)
            .map_err(|e| format!("Failed to create card execution: {}", e))?;

        // Emit CardExecutionStarted
        emit(app_handle, LeafEvent::CardExecutionStarted(card_exec.clone()));

        info!(
            "Running card '{}' (position {}) in stack execution {}",
            card.name, card.position, stack_exec.id
        );

        // Run the card's program via the Deno executor
        match executor.execute(card, event, project_path).await {
            Ok(result) => {
                card_exec.stdout = result.stdout;
                card_exec.stderr = result.stderr;
                card_exec.exit_code = Some(result.exit_code);
                card_exec.duration_ms = Some(result.duration_ms);
                card_exec.completed_at = Some(Utc::now());

                if result.exit_code == 0 {
                    card_exec.status = ExecutionStatus::Success;
                    stack_exec.completed_cards += 1;

                    info!(
                        "Card '{}' completed successfully in {}ms",
                        card.name, result.duration_ms
                    );
                } else {
                    card_exec.status = ExecutionStatus::Failed;
                    pipeline_failed = true;
                    stack_exec.failed_at_position = Some(card.position);
                    stack_exec.error = Some(format!(
                        "Card '{}' failed with exit code {}",
                        card.name, result.exit_code
                    ));

                    error!(
                        "Card '{}' failed with exit code {} in {}ms",
                        card.name, result.exit_code, result.duration_ms
                    );
                }
            }
            Err(e) => {
                let is_timeout = matches!(e, leaf_executor::ExecutorError::Timeout(_));
                card_exec.status = if is_timeout {
                    ExecutionStatus::Timeout
                } else {
                    ExecutionStatus::Failed
                };
                card_exec.stderr = e.to_string();
                card_exec.completed_at = Some(Utc::now());

                pipeline_failed = true;
                stack_exec.failed_at_position = Some(card.position);
                stack_exec.error = Some(format!("Card '{}' error: {}", card.name, e));

                error!("Card '{}' execution error: {}", card.name, e);
            }
        }

        // Update CardExecution in DB
        if let Err(e) = db.update_card_execution(&card_exec) {
            warn!("Failed to update card execution: {}", e);
        }

        // Emit CardExecutionCompleted
        emit(
            app_handle,
            LeafEvent::CardExecutionCompleted {
                card_execution_id: card_exec.id,
                status: card_exec.status.clone(),
                exit_code: card_exec.exit_code,
            },
        );

        // Emit progress
        emit(
            app_handle,
            LeafEvent::StackExecutionProgress {
                stack_execution_id: stack_exec.id,
                completed_cards: stack_exec.completed_cards,
                card_count: stack_exec.card_count,
            },
        );

        // If a card failed, stop the pipeline
        if pipeline_failed {
            break;
        }
    }

    // Finalize StackExecution
    let duration_ms = start.elapsed().as_millis() as u64;
    stack_exec.duration_ms = Some(duration_ms);
    stack_exec.completed_at = Some(Utc::now());
    stack_exec.status = if pipeline_failed {
        ExecutionStatus::Failed
    } else {
        ExecutionStatus::Success
    };

    // Update StackExecution in DB
    if let Err(e) = db.update_stack_execution(&stack_exec) {
        warn!("Failed to update stack execution: {}", e);
    }

    // Emit StackExecutionCompleted
    emit(
        app_handle,
        LeafEvent::StackExecutionCompleted {
            stack_execution_id: stack_exec.id,
            status: stack_exec.status.clone(),
        },
    );

    info!(
        "Stack execution {} completed with status {:?} in {}ms ({}/{} cards)",
        stack_exec.id, stack_exec.status, duration_ms, stack_exec.completed_cards, stack_exec.card_count
    );

    Ok(stack_exec)
}

fn emit(app_handle: &AppHandle, event: LeafEvent) {
    if let Err(e) = app_handle.emit("leaf-event", &event) {
        warn!("Failed to emit event: {}", e);
    }
}
