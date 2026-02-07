//! Event processing pipeline
//!
//! This module handles the flow of events from the file watcher through to
//! trigger matching, database persistence, and UI notification.
//!
//! Phase A: Trigger matching and execution spawning are stubbed out.
//! The watcher channel integration and event-to-DB pipeline are preserved.

use leaf_core::{Event, EventPayload, EventStatus, EventType, LeafEvent};
use leaf_db::{Database, EventQueries, StackQueries};
use leaf_watcher::{WatchEvent, WatchEventKind};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
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
        // TODO: Phase C - re-add executor for stack execution
        info!("Event processor created (execution stubbed for Phase A)");

        Self {
            event_rx,
            app_handle,
            project_id,
            db,
            project_path: Arc::new(project_path),
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
                    tracing::error!("Failed to process event: {}", e);
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

        // TODO: Phase C - Find matching stacks using TriggerConfig::evaluate
        let matched_stacks = self.find_matching_stacks(&event_type, &payload)?;
        event.matched_stacks = matched_stacks.clone();

        // Persist to database
        self.db
            .create_event(&event)
            .map_err(|e| format!("Failed to persist event: {}", e))?;

        info!(
            "Created event {} with {} matched stacks",
            event.id,
            event.matched_stacks.len()
        );

        // Emit event created to UI
        self.emit_leaf_event(LeafEvent::EventCreated(event.clone()));

        // If there are matched stacks, emit processing event
        if !matched_stacks.is_empty() {
            self.emit_leaf_event(LeafEvent::EventProcessing {
                event_id: event.id,
                matched_stacks: matched_stacks.clone(),
            });

            // Update event status to processing
            self.db
                .update_event_status(event.id, EventStatus::Processing)
                .map_err(|e| format!("Failed to update event status: {}", e))?;

            // TODO: Phase C - Trigger stack executions for each matched stack
            warn!("Stack execution not yet implemented (Phase C) - {} stacks matched but not executed", matched_stacks.len());

            // Mark as completed for now
            self.db
                .update_event_status(event.id, EventStatus::Completed)
                .map_err(|e| format!("Failed to update event status: {}", e))?;

            self.emit_leaf_event(LeafEvent::EventCompleted {
                event_id: event.id,
                status: EventStatus::Completed,
            });
        } else {
            // No matching stacks - mark as completed
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

    /// Find stacks that match the given event
    ///
    /// Uses TriggerConfig::evaluate for self-evaluating triggers per the
    /// Concepts & Synchronizations model.
    fn find_matching_stacks(
        &self,
        event_type: &EventType,
        event_payload: &EventPayload,
    ) -> Result<Vec<Uuid>, String> {
        let stacks = self
            .db
            .list_enabled_stacks(self.project_id)
            .map_err(|e| format!("Failed to list stacks: {}", e))?;

        let mut matched = Vec::new();

        for stack in stacks {
            // Use TriggerConfig's self-evaluating method
            if stack.trigger.evaluate(event_type, event_payload, &self.project_path) {
                debug!("Stack '{}' matches event", stack.name);
                matched.push(stack.id);
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
