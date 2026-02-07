//! Card execution database operations

use leaf_core::{CardExecution, ExecutionStatus, LeafError, Result};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for card execution operations
pub trait CardExecutionQueries {
    fn create_card_execution(&self, execution: &CardExecution) -> Result<()>;
    fn get_card_execution(&self, id: Uuid) -> Result<Option<CardExecution>>;
    fn list_card_executions_for_stack_execution(&self, stack_execution_id: Uuid) -> Result<Vec<CardExecution>>;
    fn update_card_execution(&self, execution: &CardExecution) -> Result<()>;
}

impl CardExecutionQueries for Database {
    fn create_card_execution(&self, execution: &CardExecution) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO card_executions (id, stack_execution_id, card_id, position, status, stdout, stderr, exit_code, started_at, completed_at, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                execution.id.to_string(),
                execution.stack_execution_id.to_string(),
                execution.card_id.to_string(),
                execution.position,
                serde_json::to_string(&execution.status)?,
                execution.stdout,
                execution.stderr,
                execution.exit_code,
                execution.started_at.to_rfc3339(),
                execution.completed_at.map(|dt| dt.to_rfc3339()),
                execution.duration_ms.map(|d| d as i64),
            ],
        )?;
        Ok(())
    }

    fn get_card_execution(&self, id: Uuid) -> Result<Option<CardExecution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, stack_execution_id, card_id, position, status, stdout, stderr, exit_code, started_at, completed_at, duration_ms
             FROM card_executions WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_card_execution);

        match result {
            Ok(exec) => Ok(Some(exec)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_card_executions_for_stack_execution(&self, stack_execution_id: Uuid) -> Result<Vec<CardExecution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, stack_execution_id, card_id, position, status, stdout, stderr, exit_code, started_at, completed_at, duration_ms
             FROM card_executions WHERE stack_execution_id = ?1 ORDER BY position ASC",
        )?;

        let executions = stmt
            .query_map(params![stack_execution_id.to_string()], row_to_card_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }

    fn update_card_execution(&self, execution: &CardExecution) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE card_executions SET status = ?2, stdout = ?3, stderr = ?4, exit_code = ?5, completed_at = ?6, duration_ms = ?7
             WHERE id = ?1",
            params![
                execution.id.to_string(),
                serde_json::to_string(&execution.status)?,
                execution.stdout,
                execution.stderr,
                execution.exit_code,
                execution.completed_at.map(|dt| dt.to_rfc3339()),
                execution.duration_ms.map(|d| d as i64),
            ],
        )?;
        Ok(())
    }
}

fn row_to_card_execution(row: &Row) -> rusqlite::Result<CardExecution> {
    let id: String = row.get(0)?;
    let stack_execution_id: String = row.get(1)?;
    let card_id: String = row.get(2)?;
    let status: String = row.get(4)?;
    let started_at: String = row.get(8)?;
    let completed_at: Option<String> = row.get(9)?;
    let duration_ms: Option<i64> = row.get(10)?;

    Ok(CardExecution {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        stack_execution_id: Uuid::parse_str(&stack_execution_id).unwrap_or_default(),
        card_id: Uuid::parse_str(&card_id).unwrap_or_default(),
        position: row.get(3)?,
        status: serde_json::from_str(&status).unwrap_or(ExecutionStatus::Pending),
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
        duration_ms: duration_ms.map(|d| d as u64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use leaf_core::{Card, Stack, StackExecution};
    use crate::{CardQueries, StackQueries, StackExecutionQueries};

    #[test]
    fn test_card_execution_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Set up stack + card + stack execution
        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        let mut card = Card::new(stack.id, "Test Card", "");
        card.program_path = "programs/test".to_string();
        db.create_card(&card).unwrap();

        let stack_exec = StackExecution::new(stack.id, None, 1);
        db.create_stack_execution(&stack_exec).unwrap();

        // Create card execution
        let mut card_exec = CardExecution::new(stack_exec.id, card.id, 0);
        card_exec.status = ExecutionStatus::Running;
        db.create_card_execution(&card_exec).unwrap();

        // Read
        let fetched = db.get_card_execution(card_exec.id).unwrap().unwrap();
        assert_eq!(fetched.card_id, card.id);
        assert_eq!(fetched.stack_execution_id, stack_exec.id);
        assert_eq!(fetched.status, ExecutionStatus::Running);

        // Update
        let mut updated = fetched;
        updated.status = ExecutionStatus::Success;
        updated.stdout = "Hello, world!\n".to_string();
        updated.exit_code = Some(0);
        updated.completed_at = Some(chrono::Utc::now());
        updated.duration_ms = Some(250);
        db.update_card_execution(&updated).unwrap();

        let fetched = db.get_card_execution(card_exec.id).unwrap().unwrap();
        assert_eq!(fetched.status, ExecutionStatus::Success);
        assert_eq!(fetched.stdout, "Hello, world!\n");
        assert_eq!(fetched.exit_code, Some(0));
        assert_eq!(fetched.duration_ms, Some(250));
    }

    #[test]
    fn test_list_card_executions_ordered_by_position() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        let mut card1 = Card::new(stack.id, "Card 1", "");
        card1.position = 0;
        db.create_card(&card1).unwrap();

        let mut card2 = Card::new(stack.id, "Card 2", "");
        card2.position = 1;
        db.create_card(&card2).unwrap();

        let stack_exec = StackExecution::new(stack.id, None, 2);
        db.create_stack_execution(&stack_exec).unwrap();

        // Create in reverse order
        let exec2 = CardExecution::new(stack_exec.id, card2.id, 1);
        db.create_card_execution(&exec2).unwrap();

        let exec1 = CardExecution::new(stack_exec.id, card1.id, 0);
        db.create_card_execution(&exec1).unwrap();

        // Should be ordered by position ASC
        let execs = db.list_card_executions_for_stack_execution(stack_exec.id).unwrap();
        assert_eq!(execs.len(), 2);
        assert_eq!(execs[0].position, 0);
        assert_eq!(execs[1].position, 1);
    }
}
