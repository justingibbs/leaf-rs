//! Execution database operations

use leaf_core::{Execution, ExecutionStatus, LeafError, Result};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for execution operations
pub trait ExecutionQueries {
    fn create_execution(&self, execution: &Execution) -> Result<()>;
    fn get_execution(&self, id: Uuid) -> Result<Option<Execution>>;
    fn list_executions_for_card(&self, card_id: Uuid, limit: Option<u32>) -> Result<Vec<Execution>>;
    fn list_executions_for_event(&self, event_id: Uuid) -> Result<Vec<Execution>>;
    fn update_execution(&self, execution: &Execution) -> Result<()>;
    fn list_running_executions(&self) -> Result<Vec<Execution>>;
}

impl ExecutionQueries for Database {
    fn create_execution(&self, execution: &Execution) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO executions (id, card_id, event_id, status, attempt, stdout, stderr, exit_code, started_at, completed_at, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                execution.id.to_string(),
                execution.card_id.to_string(),
                execution.event_id.map(|id| id.to_string()),
                serde_json::to_string(&execution.status)?,
                execution.attempt,
                execution.stdout,
                execution.stderr,
                execution.exit_code,
                execution.started_at.to_rfc3339(),
                execution.completed_at.map(|dt| dt.to_rfc3339()),
                execution.duration_ms,
            ],
        )?;
        Ok(())
    }

    fn get_execution(&self, id: Uuid) -> Result<Option<Execution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, card_id, event_id, status, attempt, stdout, stderr, exit_code, started_at, completed_at, duration_ms
             FROM executions WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_execution);

        match result {
            Ok(execution) => Ok(Some(execution)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_executions_for_card(&self, card_id: Uuid, limit: Option<u32>) -> Result<Vec<Execution>> {
        let conn = self.conn()?;
        let limit = limit.unwrap_or(50);
        let mut stmt = conn.prepare(
            "SELECT id, card_id, event_id, status, attempt, stdout, stderr, exit_code, started_at, completed_at, duration_ms
             FROM executions WHERE card_id = ?1 ORDER BY started_at DESC LIMIT ?2",
        )?;

        let executions = stmt
            .query_map(params![card_id.to_string(), limit], row_to_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }

    fn list_executions_for_event(&self, event_id: Uuid) -> Result<Vec<Execution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, card_id, event_id, status, attempt, stdout, stderr, exit_code, started_at, completed_at, duration_ms
             FROM executions WHERE event_id = ?1 ORDER BY started_at ASC",
        )?;

        let executions = stmt
            .query_map(params![event_id.to_string()], row_to_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }

    fn update_execution(&self, execution: &Execution) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE executions SET status = ?2, attempt = ?3, stdout = ?4, stderr = ?5, exit_code = ?6, completed_at = ?7, duration_ms = ?8
             WHERE id = ?1",
            params![
                execution.id.to_string(),
                serde_json::to_string(&execution.status)?,
                execution.attempt,
                execution.stdout,
                execution.stderr,
                execution.exit_code,
                execution.completed_at.map(|dt| dt.to_rfc3339()),
                execution.duration_ms,
            ],
        )?;
        Ok(())
    }

    fn list_running_executions(&self) -> Result<Vec<Execution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, card_id, event_id, status, attempt, stdout, stderr, exit_code, started_at, completed_at, duration_ms
             FROM executions WHERE status = '\"running\"' ORDER BY started_at ASC",
        )?;

        let executions = stmt
            .query_map([], row_to_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }
}

fn row_to_execution(row: &Row) -> rusqlite::Result<Execution> {
    let id: String = row.get(0)?;
    let card_id: String = row.get(1)?;
    let event_id: Option<String> = row.get(2)?;
    let status: String = row.get(3)?;
    let started_at: String = row.get(8)?;
    let completed_at: Option<String> = row.get(9)?;

    Ok(Execution {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        card_id: Uuid::parse_str(&card_id).unwrap_or_default(),
        event_id: event_id.and_then(|s| Uuid::parse_str(&s).ok()),
        status: serde_json::from_str(&status).unwrap_or(ExecutionStatus::Pending),
        attempt: row.get(4)?,
        stdout: row.get(5)?,
        stderr: row.get(6)?,
        exit_code: row.get(7)?,
        started_at: chrono::DateTime::parse_from_rfc3339(&started_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        completed_at: completed_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        }),
        duration_ms: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use leaf_core::Card;
    use crate::CardQueries;

    #[test]
    fn test_execution_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create a card first (foreign key)
        let card = Card::new(project_id, "Test Card", "");
        db.create_card(&card).unwrap();

        // Create execution
        let execution = Execution::new(card.id, None);
        db.create_execution(&execution).unwrap();

        // Read
        let fetched = db.get_execution(execution.id).unwrap().unwrap();
        assert_eq!(fetched.card_id, card.id);
        assert_eq!(fetched.status, ExecutionStatus::Pending);

        // Update
        let mut updated = fetched;
        updated.status = ExecutionStatus::Success;
        updated.stdout = "Hello, world!".to_string();
        updated.exit_code = Some(0);
        updated.completed_at = Some(chrono::Utc::now());
        db.update_execution(&updated).unwrap();

        let fetched = db.get_execution(execution.id).unwrap().unwrap();
        assert_eq!(fetched.status, ExecutionStatus::Success);
        assert_eq!(fetched.stdout, "Hello, world!");

        // List for card
        let executions = db.list_executions_for_card(card.id, None).unwrap();
        assert_eq!(executions.len(), 1);
    }
}
