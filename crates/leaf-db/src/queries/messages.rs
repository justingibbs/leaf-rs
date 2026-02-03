//! Message database operations

#[cfg_attr(not(test), allow(unused_imports))]
use leaf_core::{LeafError, Message, MessageRole, Result, ToolCall};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for message operations
pub trait MessageQueries {
    fn create_message(&self, message: &Message) -> Result<()>;
    fn get_message(&self, id: Uuid) -> Result<Option<Message>>;
    fn list_messages(&self, session_id: Uuid) -> Result<Vec<Message>>;
    fn delete_messages_for_session(&self, session_id: Uuid) -> Result<()>;
}

impl MessageQueries for Database {
    fn create_message(&self, message: &Message) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, tool_calls, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                message.id.to_string(),
                message.session_id.to_string(),
                serde_json::to_string(&message.role)?,
                message.content,
                serde_json::to_string(&message.tool_calls)?,
                message.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_message(&self, id: Uuid) -> Result<Option<Message>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, tool_calls, created_at
             FROM messages WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_message);

        match result {
            Ok(message) => Ok(Some(message)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_messages(&self, session_id: Uuid) -> Result<Vec<Message>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, tool_calls, created_at
             FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;

        let messages = stmt
            .query_map(params![session_id.to_string()], row_to_message)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(messages)
    }

    fn delete_messages_for_session(&self, session_id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
        Ok(())
    }
}

fn row_to_message(row: &Row) -> rusqlite::Result<Message> {
    let id: String = row.get(0)?;
    let session_id: String = row.get(1)?;
    let role: String = row.get(2)?;
    let tool_calls: String = row.get(4)?;
    let created_at: String = row.get(5)?;

    Ok(Message {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        session_id: Uuid::parse_str(&session_id).unwrap_or_default(),
        role: serde_json::from_str(&role).unwrap_or(MessageRole::User),
        content: row.get(3)?,
        tool_calls: serde_json::from_str(&tool_calls).unwrap_or_default(),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionQueries;
    use leaf_core::ChatSession;

    #[test]
    fn test_message_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create session first (foreign key)
        let session = ChatSession::new(project_id, "Test Session");
        db.create_session(&session).unwrap();

        // Create message
        let message = Message::new(session.id, MessageRole::User, "Hello, AI!");
        db.create_message(&message).unwrap();

        // Read
        let fetched = db.get_message(message.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Hello, AI!");
        assert_eq!(fetched.role, MessageRole::User);

        // Create assistant response
        let response = Message::new(
            session.id,
            MessageRole::Assistant,
            "Hello! How can I help you?",
        );
        db.create_message(&response).unwrap();

        // List messages
        let messages = db.list_messages(session.id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[1].role, MessageRole::Assistant);

        // Delete messages
        db.delete_messages_for_session(session.id).unwrap();
        let messages = db.list_messages(session.id).unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_message_with_tool_calls() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let session = ChatSession::new(project_id, "Test Session");
        db.create_session(&session).unwrap();

        let mut message = Message::new(session.id, MessageRole::Assistant, "Let me create a card.");
        message.tool_calls = vec![ToolCall {
            id: "call_123".to_string(),
            name: "create_card".to_string(),
            arguments: serde_json::json!({"name": "Test Card"}),
            result: Some(serde_json::json!({"id": "card_456"})),
        }];
        db.create_message(&message).unwrap();

        let fetched = db.get_message(message.id).unwrap().unwrap();
        assert_eq!(fetched.tool_calls.len(), 1);
        assert_eq!(fetched.tool_calls[0].name, "create_card");
    }
}
