//! Stack execution database operations

use leaf_core::{ExecutionStatus, LeafError, Result, StackExecution};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for stack execution operations
pub trait StackExecutionQueries {
    fn create_stack_execution(&self, execution: &StackExecution) -> Result<()>;
    fn get_stack_execution(&self, id: Uuid) -> Result<Option<StackExecution>>;
    fn list_stack_executions(&self, stack_id: Uuid, limit: Option<u32>) -> Result<Vec<StackExecution>>;
    fn update_stack_execution(&self, execution: &StackExecution) -> Result<()>;
    fn list_stack_executions_for_event(&self, event_id: Uuid) -> Result<Vec<StackExecution>>;
    fn list_stack_executions_for_project(&self, project_id: Uuid, limit: Option<u32>) -> Result<Vec<StackExecution>>;
}

impl StackExecutionQueries for Database {
    fn create_stack_execution(&self, execution: &StackExecution) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO stack_executions (id, stack_id, event_id, status, card_count, completed_cards, failed_at_position, error, started_at, completed_at, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                execution.id.to_string(),
                execution.stack_id.to_string(),
                execution.event_id.map(|id| id.to_string()),
                serde_json::to_string(&execution.status)?,
                execution.card_count,
                execution.completed_cards,
                execution.failed_at_position,
                execution.error,
                execution.started_at.to_rfc3339(),
                execution.completed_at.map(|dt| dt.to_rfc3339()),
                execution.duration_ms.map(|d| d as i64),
            ],
        )?;
        Ok(())
    }

    fn get_stack_execution(&self, id: Uuid) -> Result<Option<StackExecution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, stack_id, event_id, status, card_count, completed_cards, failed_at_position, error, started_at, completed_at, duration_ms
             FROM stack_executions WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_stack_execution);

        match result {
            Ok(exec) => Ok(Some(exec)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_stack_executions(&self, stack_id: Uuid, limit: Option<u32>) -> Result<Vec<StackExecution>> {
        let conn = self.conn()?;
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT id, stack_id, event_id, status, card_count, completed_cards, failed_at_position, error, started_at, completed_at, duration_ms
             FROM stack_executions WHERE stack_id = ?1 ORDER BY started_at DESC LIMIT ?2",
        )?;

        let executions = stmt
            .query_map(params![stack_id.to_string(), limit], row_to_stack_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }

    fn update_stack_execution(&self, execution: &StackExecution) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE stack_executions SET status = ?2, completed_cards = ?3, failed_at_position = ?4, error = ?5, completed_at = ?6, duration_ms = ?7
             WHERE id = ?1",
            params![
                execution.id.to_string(),
                serde_json::to_string(&execution.status)?,
                execution.completed_cards,
                execution.failed_at_position,
                execution.error,
                execution.completed_at.map(|dt| dt.to_rfc3339()),
                execution.duration_ms.map(|d| d as i64),
            ],
        )?;
        Ok(())
    }

    fn list_stack_executions_for_event(&self, event_id: Uuid) -> Result<Vec<StackExecution>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, stack_id, event_id, status, card_count, completed_cards, failed_at_position, error, started_at, completed_at, duration_ms
             FROM stack_executions WHERE event_id = ?1 ORDER BY started_at DESC",
        )?;

        let executions = stmt
            .query_map(params![event_id.to_string()], row_to_stack_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }

    fn list_stack_executions_for_project(&self, project_id: Uuid, limit: Option<u32>) -> Result<Vec<StackExecution>> {
        let conn = self.conn()?;
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT se.id, se.stack_id, se.event_id, se.status, se.card_count, se.completed_cards, se.failed_at_position, se.error, se.started_at, se.completed_at, se.duration_ms
             FROM stack_executions se
             JOIN stacks s ON se.stack_id = s.id
             WHERE s.project_id = ?1
             ORDER BY se.started_at DESC LIMIT ?2",
        )?;

        let executions = stmt
            .query_map(params![project_id.to_string(), limit], row_to_stack_execution)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(executions)
    }
}

fn row_to_stack_execution(row: &Row) -> rusqlite::Result<StackExecution> {
    let id: String = row.get(0)?;
    let stack_id: String = row.get(1)?;
    let event_id: Option<String> = row.get(2)?;
    let status: String = row.get(3)?;
    let started_at: String = row.get(8)?;
    let completed_at: Option<String> = row.get(9)?;
    let duration_ms: Option<i64> = row.get(10)?;

    Ok(StackExecution {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        stack_id: Uuid::parse_str(&stack_id).unwrap_or_default(),
        event_id: event_id.and_then(|s| Uuid::parse_str(&s).ok()),
        status: serde_json::from_str(&status).unwrap_or(ExecutionStatus::Pending),
        card_count: row.get(4)?,
        completed_cards: row.get(5)?,
        failed_at_position: row.get(6)?,
        error: row.get(7)?,
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
    use leaf_core::Stack;
    use crate::StackQueries;

    #[test]
    fn test_stack_execution_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        // Create
        let mut exec = StackExecution::new(stack.id, None, 3);
        exec.status = ExecutionStatus::Running;
        db.create_stack_execution(&exec).unwrap();

        // Read
        let fetched = db.get_stack_execution(exec.id).unwrap().unwrap();
        assert_eq!(fetched.stack_id, stack.id);
        assert_eq!(fetched.card_count, 3);
        assert_eq!(fetched.status, ExecutionStatus::Running);

        // Update
        let mut updated = fetched;
        updated.status = ExecutionStatus::Success;
        updated.completed_cards = 3;
        updated.completed_at = Some(chrono::Utc::now());
        updated.duration_ms = Some(1500);
        db.update_stack_execution(&updated).unwrap();

        let fetched = db.get_stack_execution(exec.id).unwrap().unwrap();
        assert_eq!(fetched.status, ExecutionStatus::Success);
        assert_eq!(fetched.completed_cards, 3);
        assert!(fetched.completed_at.is_some());
        assert_eq!(fetched.duration_ms, Some(1500));

        // List by stack
        let execs = db.list_stack_executions(stack.id, None).unwrap();
        assert_eq!(execs.len(), 1);
    }

    #[test]
    fn test_list_stack_executions_for_project() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        let exec = StackExecution::new(stack.id, None, 1);
        db.create_stack_execution(&exec).unwrap();

        let execs = db.list_stack_executions_for_project(project_id, None).unwrap();
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].id, exec.id);
    }

    #[test]
    fn test_list_stack_executions_for_event() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        let event_id = Uuid::new_v4();

        // Create event first (foreign key)
        use leaf_core::{Event, EventType, EventPayload};
        let event = Event {
            id: event_id,
            project_id,
            event_type: EventType::Manual,
            payload: EventPayload::Manual { input: None },
            status: leaf_core::EventStatus::Pending,
            matched_stacks: vec![],
            created_at: chrono::Utc::now(),
            processed_at: None,
        };
        use crate::EventQueries;
        db.create_event(&event).unwrap();

        let exec = StackExecution::new(stack.id, Some(event_id), 1);
        db.create_stack_execution(&exec).unwrap();

        let execs = db.list_stack_executions_for_event(event_id).unwrap();
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].event_id, Some(event_id));
    }

    #[test]
    fn test_stack_execution_with_failure() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        let mut exec = StackExecution::new(stack.id, None, 3);
        exec.status = ExecutionStatus::Failed;
        exec.completed_cards = 1;
        exec.failed_at_position = Some(1);
        exec.error = Some("Card failed with exit code 1".to_string());
        db.create_stack_execution(&exec).unwrap();

        let fetched = db.get_stack_execution(exec.id).unwrap().unwrap();
        assert_eq!(fetched.status, ExecutionStatus::Failed);
        assert_eq!(fetched.failed_at_position, Some(1));
        assert_eq!(fetched.error, Some("Card failed with exit code 1".to_string()));
    }
}
