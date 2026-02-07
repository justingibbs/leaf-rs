//! Stack execution pipeline
//!
//! Orchestrates running all cards in a stack sequentially via the Deno executor.
//! Creates StackExecution and CardExecution records and emits progress events.

use chrono::Utc;
use leaf_core::{
    Artifact, ArtifactType, CardExecution, Event, ExecutionStatus, LeafEvent, StackExecution,
};
use leaf_db::{ArtifactQueries, CardExecutionQueries, CardQueries, Database, StackExecutionQueries, StackQueries};
use leaf_executor::Executor;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

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

    // Get the stack's project_id for artifact registration
    let project_id = db
        .get_stack(stack_id)
        .map_err(|e| format!("Failed to get stack: {}", e))?
        .map(|s| s.project_id)
        .ok_or("Stack not found")?;

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

        // Snapshot project files before execution (for artifact detection)
        let before_snapshot = snapshot_project_files(project_path);

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

        // Detect artifacts created/modified by this card execution
        if card_exec.status == ExecutionStatus::Success {
            let after_snapshot = snapshot_project_files(project_path);
            register_execution_artifacts(
                &before_snapshot,
                &after_snapshot,
                project_id,
                card.id,
                stack_exec.id,
                project_path,
                db,
                app_handle,
            );
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

/// A snapshot of file modification times in the project directory
type FileSnapshot = HashMap<String, u64>;

/// Take a snapshot of all files in the project directory (relative paths → modification times).
/// Skips the .leaf directory.
fn snapshot_project_files(project_path: &Path) -> FileSnapshot {
    let mut snapshot = HashMap::new();
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') || e.depth() == 0
        })
        .filter_map(|e| e.ok())
    {
        if entry.depth() == 0 || entry.path().is_dir() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(project_path)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        snapshot.insert(relative, modified);
    }
    snapshot
}

/// Compare before/after snapshots and register new/modified files as artifacts.
fn register_execution_artifacts(
    before: &FileSnapshot,
    after: &FileSnapshot,
    project_id: Uuid,
    card_id: Uuid,
    stack_execution_id: Uuid,
    project_path: &Path,
    db: &Database,
    app_handle: &AppHandle,
) {
    let created_by = format!("card:{}", card_id);

    for (path, after_mtime) in after {
        let is_new = !before.contains_key(path);
        let is_modified = before.get(path).map(|t| t != after_mtime).unwrap_or(false);

        if !is_new && !is_modified {
            continue;
        }

        let full_path = project_path.join(path);
        let filename = full_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if is_new {
            // Check if artifact already exists (maybe registered by watcher)
            match db.get_artifact_by_path(project_id, path) {
                Ok(Some(_)) => continue,
                Ok(None) => {}
                Err(e) => {
                    warn!("Failed to check existing artifact: {}", e);
                    continue;
                }
            }

            let mut artifact =
                Artifact::new(project_id, path, &filename, ArtifactType::File, &created_by);
            artifact.created_by_execution_id = Some(stack_execution_id);
            artifact.mime_type = mime_guess::from_path(&full_path)
                .first()
                .map(|m| m.to_string());
            artifact.size_bytes = std::fs::metadata(&full_path).ok().map(|m| m.len() as i64);

            if let Err(e) = db.create_artifact(&artifact) {
                warn!("Failed to register execution artifact: {}", e);
            } else {
                debug!("Registered new execution artifact: {}", path);
                emit(app_handle, LeafEvent::ArtifactCreated(artifact));
            }
        } else if is_modified {
            // Update existing artifact
            match db.get_artifact_by_path(project_id, path) {
                Ok(Some(existing)) => {
                    if let Err(e) = db.update_artifact_modified(project_id, path) {
                        warn!("Failed to mark artifact modified: {}", e);
                    } else {
                        emit(
                            app_handle,
                            LeafEvent::ArtifactModified {
                                artifact_id: existing.id,
                            },
                        );
                    }
                }
                Ok(None) => {
                    // Not yet tracked — register it
                    let mut artifact = Artifact::new(
                        project_id,
                        path,
                        &filename,
                        ArtifactType::File,
                        &created_by,
                    );
                    artifact.created_by_execution_id = Some(stack_execution_id);
                    artifact.mime_type = mime_guess::from_path(&full_path)
                        .first()
                        .map(|m| m.to_string());
                    artifact.size_bytes =
                        std::fs::metadata(&full_path).ok().map(|m| m.len() as i64);

                    if let Err(e) = db.create_artifact(&artifact) {
                        warn!("Failed to register modified artifact: {}", e);
                    } else {
                        debug!("Registered modified execution artifact: {}", path);
                        emit(app_handle, LeafEvent::ArtifactCreated(artifact));
                    }
                }
                Err(e) => {
                    warn!("Failed to check existing artifact: {}", e);
                }
            }
        }
    }
}
