//! Event database operations

use leaf_core::{Event, EventPayload, EventStatus, EventType, LeafError, Result};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for event operations
pub trait EventQueries {
    fn create_event(&self, event: &Event) -> Result<()>;
    fn get_event(&self, id: Uuid) -> Result<Option<Event>>;
    fn list_events(&self, project_id: Uuid, limit: Option<u32>) -> Result<Vec<Event>>;
    fn update_event_status(&self, id: Uuid, status: EventStatus) -> Result<()>;
    fn list_pending_events(&self, project_id: Uuid) -> Result<Vec<Event>>;
}

impl EventQueries for Database {
    fn create_event(&self, event: &Event) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO events (id, project_id, event_type, payload, status, matched_cards, created_at, processed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id.to_string(),
                event.project_id.to_string(),
                serde_json::to_string(&event.event_type)?,
                serde_json::to_string(&event.payload)?,
                serde_json::to_string(&event.status)?,
                serde_json::to_string(&event.matched_cards)?,
                event.created_at.to_rfc3339(),
                event.processed_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    fn get_event(&self, id: Uuid) -> Result<Option<Event>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, event_type, payload, status, matched_cards, created_at, processed_at
             FROM events WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_event);

        match result {
            Ok(event) => Ok(Some(event)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_events(&self, project_id: Uuid, limit: Option<u32>) -> Result<Vec<Event>> {
        let conn = self.conn()?;
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT id, project_id, event_type, payload, status, matched_cards, created_at, processed_at
             FROM events WHERE project_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;

        let events = stmt
            .query_map(params![project_id.to_string(), limit], row_to_event)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(events)
    }

    fn update_event_status(&self, id: Uuid, status: EventStatus) -> Result<()> {
        let conn = self.conn()?;
        let processed_at = if matches!(status, EventStatus::Completed | EventStatus::Failed) {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        conn.execute(
            "UPDATE events SET status = ?2, processed_at = ?3 WHERE id = ?1",
            params![
                id.to_string(),
                serde_json::to_string(&status)?,
                processed_at,
            ],
        )?;
        Ok(())
    }

    fn list_pending_events(&self, project_id: Uuid) -> Result<Vec<Event>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, event_type, payload, status, matched_cards, created_at, processed_at
             FROM events WHERE project_id = ?1 AND status = '\"pending\"' ORDER BY created_at ASC",
        )?;

        let events = stmt
            .query_map(params![project_id.to_string()], row_to_event)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(events)
    }
}

fn row_to_event(row: &Row) -> rusqlite::Result<Event> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let event_type: String = row.get(2)?;
    let payload: String = row.get(3)?;
    let status: String = row.get(4)?;
    let matched_cards: String = row.get(5)?;
    let created_at: String = row.get(6)?;
    let processed_at: Option<String> = row.get(7)?;

    Ok(Event {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        project_id: Uuid::parse_str(&project_id).unwrap_or_default(),
        event_type: serde_json::from_str(&event_type).unwrap_or(EventType::Manual),
        payload: serde_json::from_str(&payload).unwrap_or(EventPayload::Manual { input: None }),
        status: serde_json::from_str(&status).unwrap_or(EventStatus::Pending),
        matched_cards: serde_json::from_str(&matched_cards).unwrap_or_default(),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        processed_at: processed_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create
        let event = Event::new(
            project_id,
            EventType::FileCreated,
            EventPayload::File {
                path: "/test.pdf".to_string(),
                size: Some(1024),
                mime_type: Some("application/pdf".to_string()),
            },
        );
        db.create_event(&event).unwrap();

        // Read
        let fetched = db.get_event(event.id).unwrap().unwrap();
        assert_eq!(fetched.event_type, EventType::FileCreated);

        // List
        let events = db.list_events(project_id, None).unwrap();
        assert_eq!(events.len(), 1);

        // Update status
        db.update_event_status(event.id, EventStatus::Completed)
            .unwrap();
        let fetched = db.get_event(event.id).unwrap().unwrap();
        assert_eq!(fetched.status, EventStatus::Completed);
        assert!(fetched.processed_at.is_some());
    }

    #[test]
    fn test_list_pending_events() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let event1 = Event::new(
            project_id,
            EventType::Manual,
            EventPayload::Manual { input: None },
        );
        db.create_event(&event1).unwrap();

        let event2 = Event::new(
            project_id,
            EventType::Manual,
            EventPayload::Manual { input: None },
        );
        db.create_event(&event2).unwrap();
        db.update_event_status(event2.id, EventStatus::Completed)
            .unwrap();

        let pending = db.list_pending_events(project_id).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, event1.id);
    }
}
