//! Event processing pipeline
//!
//! This module handles the flow of events from the file watcher through to
//! card matching, database persistence, UI notification, and execution triggering.

use chrono::Utc;
use leaf_core::{Card, Event, EventPayload, EventStatus, EventType, Execution, ExecutionStatus, LeafEvent};
use leaf_db::{CardQueries, Database, EventQueries, ExecutionQueries};
use leaf_executor::Executor;
use leaf_watcher::{WatchEvent, WatchEventKind};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Event processor that handles the lifecycle of file events
pub struct EventProcessor {
    /// Receiver for watch events
    event_rx: mpsc::Receiver<WatchEvent>,
    /// Tauri app handle for emitting events
    app_handle: AppHandle,
    /// Project ID we're processing events for
    project_id: Uuid,
    /// Database connection
    db: Database,
    /// Project root path for resolving relative paths
    project_path: Arc<std::path::PathBuf>,
    /// Executor for running card programs (if available)
    executor: Option<Executor>,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(
        event_rx: mpsc::Receiver<WatchEvent>,
        app_handle: AppHandle,
        project_id: Uuid,
        db: Database,
        project_path: std::path::PathBuf,
    ) -> Self {
        // Try to create executor
        let executor = Executor::new().ok();
        if executor.is_some() {
            info!("Event processor: Executor available for card execution");
        } else {
            warn!("Event processor: Executor not available, cards will not run automatically");
        }

        Self {
            event_rx,
            app_handle,
            project_id,
            db,
            project_path: Arc::new(project_path),
            executor,
        }
    }

    /// Start processing events
    ///
    /// This spawns a background task that processes events until the channel closes.
    pub fn start(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Event processor started for project {}", self.project_id);

            while let Some(watch_event) = self.event_rx.recv().await {
                if let Err(e) = self.process_event(watch_event).await {
                    error!("Failed to process event: {}", e);
                }
            }

            info!("Event processor stopped for project {}", self.project_id);
        })
    }

    /// Process a single watch event
    async fn process_event(&self, watch_event: WatchEvent) -> Result<(), String> {
        debug!(
            "Processing event: {:?} for {}",
            watch_event.kind,
            watch_event.path.display()
        );

        // Emit file detected event to UI
        self.emit_leaf_event(LeafEvent::FileDetected {
            project_id: self.project_id,
            path: watch_event.path.to_string_lossy().to_string(),
            event_type: match watch_event.kind {
                WatchEventKind::Created => "file_created".to_string(),
                WatchEventKind::Modified => "file_modified".to_string(),
            },
        });

        // Convert to LEAF event type
        let event_type = match watch_event.kind {
            WatchEventKind::Created => EventType::FileCreated,
            WatchEventKind::Modified => EventType::FileModified,
        };

        // Get file metadata
        let file_size = std::fs::metadata(&watch_event.path)
            .ok()
            .map(|m| m.len());

        let mime_type = mime_guess::from_path(&watch_event.path)
            .first()
            .map(|m| m.to_string());

        // Create the event payload
        let payload = EventPayload::File {
            path: watch_event.path.to_string_lossy().to_string(),
            size: file_size,
            mime_type,
        };

        // Create the LEAF event
        let mut event = Event::new(self.project_id, event_type.clone(), payload.clone());

        // Find matching cards using TriggerConfig::evaluate (Concepts & Synchronizations)
        let matched_cards = self.find_matching_cards(&event_type, &payload)?;
        event.matched_cards = matched_cards.clone();

        // Persist to database
        self.db
            .create_event(&event)
            .map_err(|e| format!("Failed to persist event: {}", e))?;

        info!(
            "Created event {} with {} matched cards",
            event.id,
            event.matched_cards.len()
        );

        // Emit event created to UI
        self.emit_leaf_event(LeafEvent::EventCreated(event.clone()));

        // If there are matched cards, emit processing event and trigger executions
        if !matched_cards.is_empty() {
            self.emit_leaf_event(LeafEvent::EventProcessing {
                event_id: event.id,
                matched_cards: matched_cards.clone(),
            });

            // Update event status to processing
            self.db
                .update_event_status(event.id, EventStatus::Processing)
                .map_err(|e| format!("Failed to update event status: {}", e))?;

            // Trigger executions for each matched card
            if let Some(ref executor) = self.executor {
                self.trigger_executions_for_event(&event, &matched_cards, executor)?;
            } else {
                warn!("No executor available - matched cards will not be executed");
            }
        } else {
            // No matching cards - mark as completed
            self.db
                .update_event_status(event.id, EventStatus::Completed)
                .map_err(|e| format!("Failed to update event status: {}", e))?;

            self.emit_leaf_event(LeafEvent::EventCompleted {
                event_id: event.id,
                status: EventStatus::Completed,
            });
        }

        Ok(())
    }

    /// Trigger executions for all matched cards
    fn trigger_executions_for_event(
        &self,
        event: &Event,
        matched_card_ids: &[Uuid],
        executor: &Executor,
    ) -> Result<(), String> {
        // Get the matched cards
        let cards: Vec<Card> = matched_card_ids
            .iter()
            .filter_map(|id| self.db.get_card(*id).ok().flatten())
            .collect();

        for card in cards {
            // Create execution record
            let mut execution = Execution::new(card.id, Some(event.id));
            execution.status = ExecutionStatus::Pending;

            // Persist to database
            if let Err(e) = self.db.create_execution(&execution) {
                error!("Failed to create execution for card {}: {}", card.id, e);
                continue;
            }

            // Emit execution started event
            self.emit_leaf_event(LeafEvent::ExecutionStarted(execution.clone()));

            // Log before moving values
            let card_name = card.name.clone();
            info!(
                "Triggered execution {} for card '{}' (event {})",
                execution.id, card_name, event.id
            );

            // Spawn async task to run execution
            let execution_id = execution.id;
            let executor = executor.clone();
            let db = self.db.clone();
            let app = self.app_handle.clone();
            let project_path = (*self.project_path).clone();
            let max_retries = card.program.max_retries;
            let event_clone = event.clone();

            tokio::spawn(async move {
                run_card_execution(
                    executor,
                    card,
                    execution_id,
                    Some(event_clone),
                    project_path,
                    db,
                    app,
                    max_retries,
                )
                .await;
            });
        }

        Ok(())
    }

    /// Find cards that match the given event
    ///
    /// Uses TriggerConfig::evaluate for self-evaluating triggers per the
    /// Concepts & Synchronizations model.
    fn find_matching_cards(
        &self,
        event_type: &EventType,
        event_payload: &EventPayload,
    ) -> Result<Vec<Uuid>, String> {
        let cards = self
            .db
            .list_enabled_cards(self.project_id)
            .map_err(|e| format!("Failed to list cards: {}", e))?;

        let mut matched = Vec::new();

        for card in cards {
            // Use TriggerConfig's self-evaluating method
            if card.trigger.evaluate(event_type, event_payload, &self.project_path) {
                debug!("Card '{}' matches event", card.name);
                matched.push(card.id);
            }
        }

        Ok(matched)
    }

    /// Emit a LEAF event to the frontend
    fn emit_leaf_event(&self, event: LeafEvent) {
        if let Err(e) = self.app_handle.emit("leaf-event", &event) {
            warn!("Failed to emit event to frontend: {}", e);
        }
    }
}

/// Run a card execution with retry logic
async fn run_card_execution(
    executor: Executor,
    card: Card,
    execution_id: Uuid,
    event: Option<Event>,
    project_path: std::path::PathBuf,
    db: Database,
    app: AppHandle,
    max_retries: u32,
) {
    let mut attempt = 1u32;

    loop {
        // Update execution status to running
        if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
            exec.status = ExecutionStatus::Running;
            exec.attempt = attempt;
            if let Err(e) = db.update_execution(&exec) {
                error!("Failed to update execution status: {}", e);
            }
        }

        // Run the execution
        let result = executor
            .execute(&card, event.as_ref(), &project_path)
            .await;

        // Process result
        match result {
            Ok(exec_result) => {
                // Update execution with result
                if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
                    exec.stdout = exec_result.stdout;
                    exec.stderr = exec_result.stderr;
                    exec.exit_code = Some(exec_result.exit_code);
                    exec.duration_ms = Some(exec_result.duration_ms);
                    exec.completed_at = Some(Utc::now());

                    if exec_result.exit_code == 0 {
                        exec.status = ExecutionStatus::Success;
                        info!("Execution {} completed successfully", execution_id);
                    } else {
                        // Check if we should retry
                        if attempt < max_retries {
                            warn!(
                                "Execution {} failed (attempt {}/{}), retrying...",
                                execution_id, attempt, max_retries
                            );
                            attempt += 1;
                            if let Err(e) = db.update_execution(&exec) {
                                error!("Failed to update execution: {}", e);
                            }
                            // Brief delay before retry
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                        exec.status = ExecutionStatus::Failed;
                        error!(
                            "Execution {} failed after {} attempts",
                            execution_id, attempt
                        );
                    }

                    if let Err(e) = db.update_execution(&exec) {
                        error!("Failed to update execution: {}", e);
                    }

                    // Emit completion event
                    if let Err(e) = app.emit(
                        "leaf-event",
                        &LeafEvent::ExecutionCompleted {
                            execution_id,
                            status: exec.status.clone(),
                            exit_code: exec.exit_code,
                        },
                    ) {
                        warn!("Failed to emit execution completed event: {}", e);
                    }
                }
                break;
            }
            Err(leaf_executor::ExecutorError::Timeout(secs)) => {
                // Update execution with timeout status
                if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
                    exec.status = ExecutionStatus::Timeout;
                    exec.stderr = format!("Execution timed out after {} seconds", secs);
                    exec.completed_at = Some(Utc::now());

                    if let Err(e) = db.update_execution(&exec) {
                        error!("Failed to update execution: {}", e);
                    }

                    // Emit completion event
                    if let Err(e) = app.emit(
                        "leaf-event",
                        &LeafEvent::ExecutionCompleted {
                            execution_id,
                            status: ExecutionStatus::Timeout,
                            exit_code: None,
                        },
                    ) {
                        warn!("Failed to emit execution completed event: {}", e);
                    }
                }
                break;
            }
            Err(e) => {
                // Check if we should retry
                if attempt < max_retries {
                    warn!(
                        "Execution {} error (attempt {}/{}): {}, retrying...",
                        execution_id, attempt, max_retries, e
                    );
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }

                // Update execution with failure
                if let Ok(Some(mut exec)) = db.get_execution(execution_id) {
                    exec.status = ExecutionStatus::Failed;
                    exec.stderr = e.to_string();
                    exec.completed_at = Some(Utc::now());

                    if let Err(e) = db.update_execution(&exec) {
                        error!("Failed to update execution: {}", e);
                    }

                    // Emit completion event
                    if let Err(e) = app.emit(
                        "leaf-event",
                        &LeafEvent::ExecutionCompleted {
                            execution_id,
                            status: ExecutionStatus::Failed,
                            exit_code: None,
                        },
                    ) {
                        warn!("Failed to emit execution completed event: {}", e);
                    }
                }
                break;
            }
        }
    }
}
