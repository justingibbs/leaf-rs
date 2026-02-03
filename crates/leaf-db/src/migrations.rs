//! Database migrations for LEAF

use leaf_core::Result;
use rusqlite::Connection;
use tracing::{debug, info};

/// Current schema version
const SCHEMA_VERSION: i32 = 1;

/// Run all migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    debug!("Current schema version: {}", current_version);

    if current_version < SCHEMA_VERSION {
        info!(
            "Running migrations from version {} to {}",
            current_version, SCHEMA_VERSION
        );

        // Run migrations in order
        if current_version < 1 {
            migration_v1(conn)?;
        }

        info!("Migrations complete");
    } else {
        debug!("Schema is up to date");
    }

    Ok(())
}

/// Initial schema - version 1
fn migration_v1(conn: &Connection) -> Result<()> {
    debug!("Running migration v1: Initial schema");

    conn.execute_batch(
        r#"
        -- Cards table
        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            trigger_config TEXT NOT NULL DEFAULT '{"type":"manual"}',
            program_config TEXT NOT NULL DEFAULT '{"language":"typescript","entrypoint":"main.ts","dependencies":[],"timeout_secs":300,"max_retries":3}',
            enabled INTEGER NOT NULL DEFAULT 1,
            session_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_cards_project_id ON cards(project_id);
        CREATE INDEX IF NOT EXISTS idx_cards_session_id ON cards(session_id);

        -- Events table
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            matched_cards TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            processed_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_events_project_id ON events(project_id);
        CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
        CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at);

        -- Executions table
        CREATE TABLE IF NOT EXISTS executions (
            id TEXT PRIMARY KEY,
            card_id TEXT NOT NULL,
            event_id TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            attempt INTEGER NOT NULL DEFAULT 1,
            stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '',
            exit_code INTEGER,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            duration_ms INTEGER,
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_executions_card_id ON executions(card_id);
        CREATE INDEX IF NOT EXISTS idx_executions_event_id ON executions(event_id);
        CREATE INDEX IF NOT EXISTS idx_executions_status ON executions(status);

        -- Chat sessions table
        CREATE TABLE IF NOT EXISTS chat_sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chat_sessions_project_id ON chat_sessions(project_id);
        CREATE INDEX IF NOT EXISTS idx_chat_sessions_status ON chat_sessions(status);

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            tool_calls TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);

        -- MCP servers table
        CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            command TEXT NOT NULL,
            args TEXT NOT NULL DEFAULT '[]',
            env TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_mcp_servers_project_id ON mcp_servers(project_id);
        "#,
    )?;

    // Record migration
    conn.execute("INSERT INTO _migrations (version) VALUES (1)", [])?;

    debug!("Migration v1 complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).expect("Migrations should succeed");

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"cards".to_string()));
        assert!(tables.contains(&"events".to_string()));
        assert!(tables.contains(&"executions".to_string()));
        assert!(tables.contains(&"chat_sessions".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"mcp_servers".to_string()));
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).expect("First migration should succeed");
        run_migrations(&conn).expect("Second migration should also succeed");
    }
}
