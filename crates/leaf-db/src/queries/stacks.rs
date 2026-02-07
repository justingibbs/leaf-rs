//! Stack database operations

use leaf_core::{LeafError, Result, Stack};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for stack operations
pub trait StackQueries {
    fn create_stack(&self, stack: &Stack) -> Result<()>;
    fn get_stack(&self, id: Uuid) -> Result<Option<Stack>>;
    fn list_stacks(&self, project_id: Uuid) -> Result<Vec<Stack>>;
    fn update_stack(&self, stack: &Stack) -> Result<()>;
    fn delete_stack(&self, id: Uuid) -> Result<()>;
    fn enable_stack(&self, id: Uuid) -> Result<()>;
    fn disable_stack(&self, id: Uuid) -> Result<()>;
    fn list_enabled_stacks(&self, project_id: Uuid) -> Result<Vec<Stack>>;
}

impl StackQueries for Database {
    fn create_stack(&self, stack: &Stack) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO stacks (id, project_id, name, description, trigger_config, enabled, source_session_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                stack.id.to_string(),
                stack.project_id.to_string(),
                stack.name,
                stack.description,
                serde_json::to_string(&stack.trigger)?,
                stack.enabled,
                stack.source_session_id.map(|id| id.to_string()),
                stack.created_at.to_rfc3339(),
                stack.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_stack(&self, id: Uuid) -> Result<Option<Stack>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, trigger_config, enabled, source_session_id, created_at, updated_at
             FROM stacks WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_stack);

        match result {
            Ok(stack) => Ok(Some(stack)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_stacks(&self, project_id: Uuid) -> Result<Vec<Stack>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, trigger_config, enabled, source_session_id, created_at, updated_at
             FROM stacks WHERE project_id = ?1 ORDER BY created_at DESC"
        )?;

        let stacks = stmt
            .query_map(params![project_id.to_string()], row_to_stack)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(stacks)
    }

    fn update_stack(&self, stack: &Stack) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE stacks SET name = ?2, description = ?3, trigger_config = ?4, enabled = ?5, source_session_id = ?6, updated_at = ?7
             WHERE id = ?1",
            params![
                stack.id.to_string(),
                stack.name,
                stack.description,
                serde_json::to_string(&stack.trigger)?,
                stack.enabled,
                stack.source_session_id.map(|id| id.to_string()),
                stack.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete_stack(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM stacks WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn enable_stack(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE stacks SET enabled = 1, updated_at = ?2 WHERE id = ?1",
            params![id.to_string(), chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn disable_stack(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE stacks SET enabled = 0, updated_at = ?2 WHERE id = ?1",
            params![id.to_string(), chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn list_enabled_stacks(&self, project_id: Uuid) -> Result<Vec<Stack>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, trigger_config, enabled, source_session_id, created_at, updated_at
             FROM stacks WHERE project_id = ?1 AND enabled = 1 ORDER BY created_at DESC"
        )?;

        let stacks = stmt
            .query_map(params![project_id.to_string()], row_to_stack)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(stacks)
    }
}

fn row_to_stack(row: &Row) -> rusqlite::Result<Stack> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let trigger_config: String = row.get(4)?;
    let source_session_id: Option<String> = row.get(6)?;
    let created_at: String = row.get(7)?;
    let updated_at: String = row.get(8)?;

    Ok(Stack {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        project_id: Uuid::parse_str(&project_id).unwrap_or_default(),
        name: row.get(2)?,
        description: row.get(3)?,
        trigger: serde_json::from_str(&trigger_config).unwrap_or_default(),
        enabled: row.get(5)?,
        source_session_id: source_session_id.and_then(|s| Uuid::parse_str(&s).ok()),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use leaf_core::TriggerConfig;

    #[test]
    fn test_stack_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create
        let mut stack = Stack::new(project_id, "Test Stack", "A test stack");
        stack.trigger = TriggerConfig::FileCreated {
            watch_path: "inbox".to_string(),
            patterns: vec!["*.csv".to_string()],
        };
        db.create_stack(&stack).unwrap();

        // Read
        let fetched = db.get_stack(stack.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test Stack");
        assert_eq!(fetched.project_id, project_id);
        match &fetched.trigger {
            TriggerConfig::FileCreated { watch_path, patterns } => {
                assert_eq!(watch_path, "inbox");
                assert_eq!(patterns, &vec!["*.csv".to_string()]);
            }
            _ => panic!("Wrong trigger type"),
        }

        // List
        let stacks = db.list_stacks(project_id).unwrap();
        assert_eq!(stacks.len(), 1);

        // Update
        let mut updated = fetched;
        updated.name = "Updated Stack".to_string();
        db.update_stack(&updated).unwrap();

        let fetched = db.get_stack(stack.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Stack");

        // Delete
        db.delete_stack(stack.id).unwrap();
        let fetched = db.get_stack(stack.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_enable_disable_stack() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack = Stack::new(project_id, "Test Stack", "");
        db.create_stack(&stack).unwrap();

        // Disable
        db.disable_stack(stack.id).unwrap();
        let fetched = db.get_stack(stack.id).unwrap().unwrap();
        assert!(!fetched.enabled);

        // Enable
        db.enable_stack(stack.id).unwrap();
        let fetched = db.get_stack(stack.id).unwrap().unwrap();
        assert!(fetched.enabled);
    }

    #[test]
    fn test_list_enabled_stacks() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let stack1 = Stack::new(project_id, "Enabled", "");
        db.create_stack(&stack1).unwrap();

        let mut stack2 = Stack::new(project_id, "Disabled", "");
        stack2.enabled = false;
        db.create_stack(&stack2).unwrap();

        let enabled = db.list_enabled_stacks(project_id).unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "Enabled");
    }
}
