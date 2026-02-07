//! LEAF Database - SQLite storage layer for LEAF
//!
//! This crate provides database operations for all LEAF entities:
//! - Stacks, Cards, Events, Executions
//! - Chat sessions and messages
//! - MCP server configurations
//! - Artifacts

pub mod migrations;
pub mod queries;

use std::path::Path;
use std::sync::{Arc, Mutex};

use leaf_core::{LeafError, Result};
use rusqlite::Connection;
use tracing::{debug, info};

/// Database handle for LEAF operations
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Opening database at: {}", path.display());

        let conn = Connection::open(path)?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // Run migrations
        db.migrate()?;

        info!("Database initialized at: {}", path.display());
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        debug!("Creating in-memory database");

        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.migrate()?;

        info!("In-memory database initialized");
        Ok(db)
    }

    /// Run database migrations
    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| LeafError::database(e.to_string()))?;
        migrations::run_migrations(&conn)?;
        Ok(())
    }

    /// Get a reference to the connection for queries
    pub(crate) fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| LeafError::database(format!("Failed to acquire lock: {}", e)))
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

// Re-export query modules
pub use queries::artifacts::ArtifactQueries;
pub use queries::card_executions::CardExecutionQueries;
pub use queries::cards::CardQueries;
pub use queries::events::EventQueries;
pub use queries::mcp::McpServerQueries;
pub use queries::messages::MessageQueries;
pub use queries::sessions::SessionQueries;
pub use queries::stack_executions::StackExecutionQueries;
pub use queries::stacks::StackQueries;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database() {
        let db = Database::in_memory().expect("Failed to create database");
        assert!(db.conn().is_ok());
    }

    #[test]
    fn test_database_clone() {
        let db = Database::in_memory().expect("Failed to create database");
        let db2 = db.clone();
        // Both should work
        assert!(db.conn().is_ok());
        assert!(db2.conn().is_ok());
    }
}
