//! Application state management

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use leaf_core::{AppConfig, LeafEvent, Project, ProjectConfig, Result, WatchPath};
use leaf_db::Database;
use leaf_watcher::{FileWatcher, WatchConfig, WatchEvent};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::events::EventProcessor;

/// Application state shared across Tauri commands
pub struct AppState {
    /// Application configuration
    pub config: Arc<RwLock<AppConfig>>,
    /// Currently open project (if any)
    pub current_project: Arc<RwLock<Option<OpenProject>>>,
}

/// A currently open project with its database connection and watcher
pub struct OpenProject {
    pub project: Project,
    pub config: ProjectConfig,
    pub db: Database,
    pub path: PathBuf,
    /// File watcher for this project (if running)
    pub watcher: Option<WatcherHandle>,
}

/// Handle to a running file watcher
pub struct WatcherHandle {
    /// The file watcher instance
    watcher: FileWatcher,
    /// Sender for stopping the event processor
    _event_processor_handle: tokio::task::JoinHandle<()>,
}

impl WatcherHandle {
    /// Get the list of currently watched paths
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        self.watcher.watched_paths()
    }
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Result<Self> {
        let config = AppConfig::default();

        // Ensure data directory exists
        if !config.data_dir.exists() {
            std::fs::create_dir_all(&config.data_dir)?;
            debug!("Created data directory: {}", config.data_dir.display());
        }

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            current_project: Arc::new(RwLock::new(None)),
        })
    }

    /// Open a project at the given path
    ///
    /// This is the low-level method that just opens the project without starting the watcher.
    /// For normal usage, prefer `open_project_and_watch` which follows the spec's
    /// synchronization: `Project.open() -> Watcher.start()`.
    fn open_project_internal(&self, path: PathBuf) -> Result<Project> {
        let leaf_dir = path.join(".leaf");

        // Create .leaf directory if it doesn't exist
        if !leaf_dir.exists() {
            std::fs::create_dir_all(&leaf_dir)?;
            info!("Created .leaf directory: {}", leaf_dir.display());
        }

        // Open or create database
        let db_path = leaf_dir.join("leaf.db");
        let db = Database::open(&db_path)?;

        // Load or create project config
        let config_path = leaf_dir.join("config.json");
        let project_config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| {
                ProjectConfig::new(
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Untitled"),
                )
            })
        } else {
            let config = ProjectConfig::new(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled"),
            );
            let content = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;
            config
        };

        // Create project
        let project = Project::new(&project_config.name, path.to_string_lossy().to_string());

        // Store in state
        let open_project = OpenProject {
            project: project.clone(),
            config: project_config,
            db,
            path,
            watcher: None,
        };

        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;
        *current = Some(open_project);

        info!("Opened project: {}", project.name);
        Ok(project)
    }

    /// Open a project and automatically start the file watcher
    ///
    /// This implements the spec's synchronization rule:
    /// `Project.open(id) -> Watcher.start(project.path)`
    ///
    /// The watcher will monitor all paths configured in the project's watch_paths.
    pub fn open_project(&self, path: PathBuf, app_handle: AppHandle) -> Result<Project> {
        // First, open the project
        let project = self.open_project_internal(path)?;

        // Then, start the watcher (per spec: Project.open -> Watcher.start)
        match self.start_watcher(app_handle) {
            Ok(paths) => {
                if paths.is_empty() {
                    info!("Project opened, no watch paths configured");
                } else {
                    info!("Project opened, watching {} paths", paths.len());
                }
            }
            Err(e) => {
                // Log the error but don't fail the project open
                // The watcher can be started manually later
                warn!("Project opened but failed to start watcher: {}", e);
            }
        }

        Ok(project)
    }

    /// Close the current project
    pub fn close_project(&self) -> Result<()> {
        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;
        if let Some(project) = current.take() {
            info!("Closed project: {}", project.project.name);
        }
        Ok(())
    }

    /// Get the current project (if any)
    pub fn get_current_project(&self) -> Result<Option<Project>> {
        let current = self
            .current_project
            .read()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;
        Ok(current.as_ref().map(|p| p.project.clone()))
    }

    /// Get the database for the current project
    pub fn get_db(&self) -> Result<Database> {
        let current = self
            .current_project
            .read()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;
        current
            .as_ref()
            .map(|p| p.db.clone())
            .ok_or_else(|| leaf_core::LeafError::Other("No project open".to_string()))
    }

    /// Start the file watcher for the current project
    pub fn start_watcher(&self, app_handle: AppHandle) -> Result<Vec<PathBuf>> {
        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        let open_project = current
            .as_mut()
            .ok_or_else(|| leaf_core::LeafError::Other("No project open".to_string()))?;

        // Don't start if already running
        if open_project.watcher.is_some() {
            return Ok(open_project
                .watcher
                .as_ref()
                .map(|w| w.watched_paths())
                .unwrap_or_default());
        }

        // Create event channel
        let (event_tx, event_rx) = mpsc::channel::<WatchEvent>(100);

        // Create the file watcher
        let mut watcher = FileWatcher::new(event_tx)
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        // Watch paths from config
        let mut watched_paths = Vec::new();
        for watch_path in &open_project.config.watch_paths {
            if !watch_path.enabled {
                continue;
            }

            let full_path = resolve_watch_path(&open_project.path, &watch_path.path);

            if !full_path.exists() {
                warn!("Watch path does not exist: {}", full_path.display());
                continue;
            }

            let config = WatchConfig {
                path: full_path.clone(),
                patterns: watch_path.patterns.clone(),
                debounce_ms: watch_path.debounce_ms,
            };

            match watcher.watch(config) {
                Ok(()) => {
                    info!("Started watching: {}", full_path.display());
                    watched_paths.push(full_path);
                }
                Err(e) => {
                    warn!("Failed to watch {}: {}", full_path.display(), e);
                }
            }
        }

        // Create and start the event processor
        let processor = EventProcessor::new(
            event_rx,
            app_handle.clone(),
            open_project.project.id,
            open_project.db.clone(),
            open_project.path.clone(),
        );
        let processor_handle = processor.start();

        // Emit watcher started event
        if let Err(e) = app_handle.emit(
            "leaf-event",
            &LeafEvent::WatcherStarted {
                project_id: open_project.project.id,
                paths: watched_paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
            },
        ) {
            warn!("Failed to emit watcher started event: {}", e);
        }

        // Store the watcher handle
        open_project.watcher = Some(WatcherHandle {
            watcher,
            _event_processor_handle: processor_handle,
        });

        info!(
            "File watcher started with {} paths",
            watched_paths.len()
        );

        Ok(watched_paths)
    }

    /// Stop the file watcher for the current project
    pub fn stop_watcher(&self, app_handle: AppHandle) -> Result<()> {
        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        let open_project = current
            .as_mut()
            .ok_or_else(|| leaf_core::LeafError::Other("No project open".to_string()))?;

        if let Some(_watcher) = open_project.watcher.take() {
            // Dropping the watcher will stop it
            info!("File watcher stopped");

            // Emit watcher stopped event
            if let Err(e) = app_handle.emit(
                "leaf-event",
                &LeafEvent::WatcherStopped {
                    project_id: open_project.project.id,
                },
            ) {
                warn!("Failed to emit watcher stopped event: {}", e);
            }
        }

        Ok(())
    }

    /// Add a watch path to the current project
    pub fn add_watch_path(
        &self,
        _app_handle: AppHandle,
        path: String,
        patterns: Vec<String>,
    ) -> Result<()> {
        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        let open_project = current
            .as_mut()
            .ok_or_else(|| leaf_core::LeafError::Other("No project open".to_string()))?;

        // Add to config
        let watch_path = WatchPath {
            path: path.clone(),
            patterns: patterns.clone(),
            enabled: true,
            debounce_ms: 500,
        };
        open_project.config.watch_paths.push(watch_path);

        // Save config
        let config_path = open_project.path.join(".leaf").join("config.json");
        let content = serde_json::to_string_pretty(&open_project.config)?;
        std::fs::write(&config_path, content)?;

        // If watcher is running, add the path
        if let Some(ref mut watcher_handle) = open_project.watcher {
            let full_path = resolve_watch_path(&open_project.path, &path);

            if full_path.exists() {
                let config = WatchConfig {
                    path: full_path.clone(),
                    patterns,
                    debounce_ms: 500,
                };

                if let Err(e) = watcher_handle.watcher.watch(config) {
                    error!("Failed to watch path: {}", e);
                } else {
                    info!("Added watch path: {}", full_path.display());
                }
            }
        }

        Ok(())
    }

    /// Remove a watch path from the current project
    pub fn remove_watch_path(&self, _app_handle: AppHandle, path: String) -> Result<()> {
        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        let open_project = current
            .as_mut()
            .ok_or_else(|| leaf_core::LeafError::Other("No project open".to_string()))?;

        // Remove from config
        open_project
            .config
            .watch_paths
            .retain(|wp| wp.path != path);

        // Save config
        let config_path = open_project.path.join(".leaf").join("config.json");
        let content = serde_json::to_string_pretty(&open_project.config)?;
        std::fs::write(&config_path, content)?;

        // If watcher is running, remove the path
        if let Some(ref mut watcher_handle) = open_project.watcher {
            let full_path = resolve_watch_path(&open_project.path, &path);

            if let Err(e) = watcher_handle.watcher.unwatch(&full_path) {
                warn!("Failed to unwatch path: {}", e);
            } else {
                info!("Removed watch path: {}", full_path.display());
            }
        }

        Ok(())
    }

    /// Get the list of watch paths for the current project
    pub fn get_watch_paths(&self) -> Result<Vec<WatchPath>> {
        let current = self
            .current_project
            .read()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        let open_project = current
            .as_ref()
            .ok_or_else(|| leaf_core::LeafError::Other("No project open".to_string()))?;

        Ok(open_project.config.watch_paths.clone())
    }

    /// Check if the watcher is running
    pub fn is_watcher_running(&self) -> Result<bool> {
        let current = self
            .current_project
            .read()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;

        Ok(current
            .as_ref()
            .map(|p| p.watcher.is_some())
            .unwrap_or(false))
    }
}

/// Resolve a watch path relative to the project root
fn resolve_watch_path(project_path: &std::path::Path, watch_path: &str) -> PathBuf {
    let path = PathBuf::from(watch_path);
    if path.is_absolute() {
        path
    } else {
        project_path.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new().expect("Failed to create app state");
        assert!(state.get_current_project().unwrap().is_none());
    }

    #[test]
    fn test_open_close_project() {
        let state = AppState::new().expect("Failed to create app state");
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Open project using internal method (open_project requires AppHandle)
        let project = state
            .open_project_internal(temp_dir.path().to_path_buf())
            .expect("Failed to open project");
        assert!(!project.name.is_empty());

        // Verify project is open
        let current = state.get_current_project().unwrap();
        assert!(current.is_some());

        // Close project
        state.close_project().expect("Failed to close project");

        // Verify project is closed
        let current = state.get_current_project().unwrap();
        assert!(current.is_none());
    }

    #[test]
    fn test_resolve_watch_path() {
        use std::path::Path;

        let project_path = Path::new("/home/user/project");

        // Relative path
        let result = resolve_watch_path(project_path, "inbox");
        assert_eq!(result, PathBuf::from("/home/user/project/inbox"));

        // Absolute path
        let result = resolve_watch_path(project_path, "/tmp/inbox");
        assert_eq!(result, PathBuf::from("/tmp/inbox"));
    }
}
