//! Chat session database operations

use leaf_core::{ChatSession, LeafError, Result, SessionStatus};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for session operations
pub trait SessionQueries {
    fn create_session(&self, session: &ChatSession) -> Result<()>;
    fn get_session(&self, id: Uuid) -> Result<Option<ChatSession>>;
    fn list_sessions(&self, project_id: Uuid) -> Result<Vec<ChatSession>>;
    fn update_session(&self, session: &ChatSession) -> Result<()>;
    fn delete_session(&self, id: Uuid) -> Result<()>;
    fn list_active_sessions(&self, project_id: Uuid) -> Result<Vec<ChatSession>>;
}

impl SessionQueries for Database {
    fn create_session(&self, session: &ChatSession) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO chat_sessions (id, project_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id.to_string(),
                session.project_id.to_string(),
                session.title,
                serde_json::to_string(&session.status)?,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_session(&self, id: Uuid) -> Result<Option<ChatSession>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, title, status, created_at, updated_at
             FROM chat_sessions WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_session);

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_sessions(&self, project_id: Uuid) -> Result<Vec<ChatSession>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, title, status, created_at, updated_at
             FROM chat_sessions WHERE project_id = ?1 ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id.to_string()], row_to_session)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(sessions)
    }

    fn update_session(&self, session: &ChatSession) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE chat_sessions SET title = ?2, status = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                session.id.to_string(),
                session.title,
                serde_json::to_string(&session.status)?,
                session.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete_session(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM chat_sessions WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    fn list_active_sessions(&self, project_id: Uuid) -> Result<Vec<ChatSession>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, title, status, created_at, updated_at
             FROM chat_sessions WHERE project_id = ?1 AND status = '\"active\"' ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id.to_string()], row_to_session)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(sessions)
    }
}

fn row_to_session(row: &Row) -> rusqlite::Result<ChatSession> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let status: String = row.get(3)?;
    let created_at: String = row.get(4)?;
    let updated_at: String = row.get(5)?;

    Ok(ChatSession {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        project_id: Uuid::parse_str(&project_id).unwrap_or_default(),
        title: row.get(2)?,
        status: serde_json::from_str(&status).unwrap_or(SessionStatus::Active),
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

    #[test]
    fn test_session_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create
        let session = ChatSession::new(project_id, "Test Session");
        db.create_session(&session).unwrap();

        // Read
        let fetched = db.get_session(session.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Session");

        // List
        let sessions = db.list_sessions(project_id).unwrap();
        assert_eq!(sessions.len(), 1);

        // Update
        let mut updated = fetched;
        updated.title = "Updated Session".to_string();
        updated.status = SessionStatus::Completed;
        db.update_session(&updated).unwrap();

        let fetched = db.get_session(session.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Updated Session");
        assert_eq!(fetched.status, SessionStatus::Completed);

        // Delete
        db.delete_session(session.id).unwrap();
        let fetched = db.get_session(session.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_list_active_sessions() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let session1 = ChatSession::new(project_id, "Active");
        db.create_session(&session1).unwrap();

        let mut session2 = ChatSession::new(project_id, "Completed");
        session2.status = SessionStatus::Completed;
        db.create_session(&session2).unwrap();

        let active = db.list_active_sessions(project_id).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Active");
    }
}
