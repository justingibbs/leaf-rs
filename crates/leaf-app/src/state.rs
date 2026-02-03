//! Application state management

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use leaf_core::{AppConfig, Project, ProjectConfig, Result};
use leaf_db::Database;
use tracing::{debug, info};

/// Application state shared across Tauri commands
pub struct AppState {
    /// Application configuration
    pub config: Arc<RwLock<AppConfig>>,
    /// Currently open project (if any)
    pub current_project: Arc<RwLock<Option<OpenProject>>>,
}

/// A currently open project with its database connection
pub struct OpenProject {
    pub project: Project,
    pub config: ProjectConfig,
    pub db: Database,
    pub path: PathBuf,
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
    pub fn open_project(&self, path: PathBuf) -> Result<Project> {
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
        };

        let mut current = self
            .current_project
            .write()
            .map_err(|e| leaf_core::LeafError::Other(e.to_string()))?;
        *current = Some(open_project);

        info!("Opened project: {}", project.name);
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

        // Open project
        let project = state
            .open_project(temp_dir.path().to_path_buf())
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
}
