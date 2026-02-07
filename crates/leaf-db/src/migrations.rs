//! Database migrations for LEAF

use leaf_core::Result;
use rusqlite::Connection;
use tracing::{debug, info};

/// Current schema version
const SCHEMA_VERSION: i32 = 2;

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
        if current_version < 2 {
            migration_v2(conn)?;
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
        -- Cards table (v1 - will be replaced in v2)
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

        -- Executions table (v1 - will be replaced in v2)
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

/// Stack & Cards schema - version 2
///
/// Replaces the flat Card model with Stack + Cards pipeline.
/// Drops old executions and cards tables.
fn migration_v2(conn: &Connection) -> Result<()> {
    debug!("Running migration v2: Stack & Cards schema");

    conn.execute_batch(
        r#"
        -- Drop old tables (order matters for foreign keys)
        DROP TABLE IF EXISTS executions;
        DROP TABLE IF EXISTS cards;

        -- Update events table: rename matched_cards to matched_stacks
        -- SQLite doesn't support ALTER COLUMN RENAME, so we recreate
        CREATE TABLE IF NOT EXISTS events_new (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            matched_stacks TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            processed_at TEXT
        );

        INSERT OR IGNORE INTO events_new (id, project_id, event_type, payload, status, matched_stacks, created_at, processed_at)
            SELECT id, project_id, event_type, payload, status, matched_cards, created_at, processed_at FROM events;

        DROP TABLE IF EXISTS events;
        ALTER TABLE events_new RENAME TO events;

        CREATE INDEX IF NOT EXISTS idx_events_project_id ON events(project_id);
        CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
        CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at);

        -- Stacks table
        CREATE TABLE IF NOT EXISTS stacks (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            trigger_config TEXT NOT NULL DEFAULT '{"type":"manual"}',
            enabled INTEGER NOT NULL DEFAULT 1,
            source_session_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_stacks_project_id ON stacks(project_id);
        CREATE INDEX IF NOT EXISTS idx_stacks_enabled ON stacks(enabled);
        CREATE INDEX IF NOT EXISTS idx_stacks_source_session_id ON stacks(source_session_id);

        -- Cards table (new schema - belongs to a stack)
        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            stack_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            program_config TEXT NOT NULL DEFAULT '{"language":"typescript","entrypoint":"main.ts","dependencies":[],"timeout_secs":300,"max_retries":3}',
            program_path TEXT NOT NULL DEFAULT '',
            position INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_cards_stack_id ON cards(stack_id);
        CREATE INDEX IF NOT EXISTS idx_cards_position ON cards(position);

        -- Stack executions table
        CREATE TABLE IF NOT EXISTS stack_executions (
            id TEXT PRIMARY KEY,
            stack_id TEXT NOT NULL,
            event_id TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            card_count INTEGER NOT NULL DEFAULT 0,
            completed_cards INTEGER NOT NULL DEFAULT 0,
            failed_at_position INTEGER,
            error TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            duration_ms INTEGER,
            FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE,
            FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_stack_executions_stack_id ON stack_executions(stack_id);
        CREATE INDEX IF NOT EXISTS idx_stack_executions_event_id ON stack_executions(event_id);
        CREATE INDEX IF NOT EXISTS idx_stack_executions_status ON stack_executions(status);
        CREATE INDEX IF NOT EXISTS idx_stack_executions_started_at ON stack_executions(started_at);

        -- Card executions table
        CREATE TABLE IF NOT EXISTS card_executions (
            id TEXT PRIMARY KEY,
            stack_execution_id TEXT NOT NULL,
            card_id TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending',
            stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '',
            exit_code INTEGER,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            duration_ms INTEGER,
            FOREIGN KEY (stack_execution_id) REFERENCES stack_executions(id) ON DELETE CASCADE,
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_card_executions_stack_execution_id ON card_executions(stack_execution_id);
        CREATE INDEX IF NOT EXISTS idx_card_executions_card_id ON card_executions(card_id);
        CREATE INDEX IF NOT EXISTS idx_card_executions_status ON card_executions(status);

        -- Artifacts table
        CREATE TABLE IF NOT EXISTS artifacts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            path TEXT NOT NULL,
            filename TEXT NOT NULL,
            artifact_type TEXT NOT NULL DEFAULT 'file',
            mime_type TEXT,
            size_bytes INTEGER,
            created_by TEXT NOT NULL,
            created_by_execution_id TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL,
            metadata TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_artifacts_project_id ON artifacts(project_id);
        CREATE INDEX IF NOT EXISTS idx_artifacts_created_by ON artifacts(created_by);
        CREATE INDEX IF NOT EXISTS idx_artifacts_status ON artifacts(status);
        CREATE INDEX IF NOT EXISTS idx_artifacts_path ON artifacts(path);
        "#,
    )?;

    // Record migration
    conn.execute("INSERT INTO _migrations (version) VALUES (2)", [])?;

    debug!("Migration v2 complete");
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

        assert!(tables.contains(&"stacks".to_string()));
        assert!(tables.contains(&"cards".to_string()));
        assert!(tables.contains(&"stack_executions".to_string()));
        assert!(tables.contains(&"card_executions".to_string()));
        assert!(tables.contains(&"artifacts".to_string()));
        assert!(tables.contains(&"events".to_string()));
        assert!(tables.contains(&"chat_sessions".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"mcp_servers".to_string()));
        // Old executions table should NOT exist
        assert!(!tables.contains(&"executions".to_string()));
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).expect("First migration should succeed");
        run_migrations(&conn).expect("Second migration should also succeed");
    }

    #[test]
    fn test_events_table_has_matched_stacks() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).expect("Migrations should succeed");

        // Verify the events table has matched_stacks column
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(events)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(columns.contains(&"matched_stacks".to_string()));
        assert!(!columns.contains(&"matched_cards".to_string()));
    }

    #[test]
    fn test_cards_table_has_stack_id() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).expect("Migrations should succeed");

        // Verify cards table has stack_id column
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(cards)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(columns.contains(&"stack_id".to_string()));
        assert!(columns.contains(&"program_path".to_string()));
        assert!(columns.contains(&"position".to_string()));
        // Old columns should NOT exist
        assert!(!columns.contains(&"project_id".to_string()));
        assert!(!columns.contains(&"trigger_config".to_string()));
        assert!(!columns.contains(&"session_id".to_string()));
    }
}
