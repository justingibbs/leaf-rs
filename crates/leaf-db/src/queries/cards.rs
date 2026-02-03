//! Card database operations

use leaf_core::{Card, LeafError, Result};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for card operations
pub trait CardQueries {
    fn create_card(&self, card: &Card) -> Result<()>;
    fn get_card(&self, id: Uuid) -> Result<Option<Card>>;
    fn list_cards(&self, project_id: Uuid) -> Result<Vec<Card>>;
    fn update_card(&self, card: &Card) -> Result<()>;
    fn delete_card(&self, id: Uuid) -> Result<()>;
    fn list_enabled_cards(&self, project_id: Uuid) -> Result<Vec<Card>>;
}

impl CardQueries for Database {
    fn create_card(&self, card: &Card) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO cards (id, project_id, name, description, trigger_config, program_config, enabled, session_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                card.id.to_string(),
                card.project_id.to_string(),
                card.name,
                card.description,
                serde_json::to_string(&card.trigger)?,
                serde_json::to_string(&card.program)?,
                card.enabled,
                card.session_id.map(|id| id.to_string()),
                card.created_at.to_rfc3339(),
                card.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_card(&self, id: Uuid) -> Result<Option<Card>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, trigger_config, program_config, enabled, session_id, created_at, updated_at
             FROM cards WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_card);

        match result {
            Ok(card) => Ok(Some(card)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_cards(&self, project_id: Uuid) -> Result<Vec<Card>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, trigger_config, program_config, enabled, session_id, created_at, updated_at
             FROM cards WHERE project_id = ?1 ORDER BY created_at DESC"
        )?;

        let cards = stmt
            .query_map(params![project_id.to_string()], row_to_card)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(cards)
    }

    fn update_card(&self, card: &Card) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cards SET name = ?2, description = ?3, trigger_config = ?4, program_config = ?5, enabled = ?6, session_id = ?7, updated_at = ?8
             WHERE id = ?1",
            params![
                card.id.to_string(),
                card.name,
                card.description,
                serde_json::to_string(&card.trigger)?,
                serde_json::to_string(&card.program)?,
                card.enabled,
                card.session_id.map(|id| id.to_string()),
                card.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete_card(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM cards WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn list_enabled_cards(&self, project_id: Uuid) -> Result<Vec<Card>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, trigger_config, program_config, enabled, session_id, created_at, updated_at
             FROM cards WHERE project_id = ?1 AND enabled = 1 ORDER BY created_at DESC"
        )?;

        let cards = stmt
            .query_map(params![project_id.to_string()], row_to_card)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(cards)
    }
}

fn row_to_card(row: &Row) -> rusqlite::Result<Card> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let trigger_config: String = row.get(4)?;
    let program_config: String = row.get(5)?;
    let session_id: Option<String> = row.get(7)?;
    let created_at: String = row.get(8)?;
    let updated_at: String = row.get(9)?;

    Ok(Card {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        project_id: Uuid::parse_str(&project_id).unwrap_or_default(),
        name: row.get(2)?,
        description: row.get(3)?,
        trigger: serde_json::from_str(&trigger_config).unwrap_or_default(),
        program: serde_json::from_str(&program_config).unwrap_or_default(),
        enabled: row.get(6)?,
        session_id: session_id.and_then(|s| Uuid::parse_str(&s).ok()),
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
    fn test_card_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create
        let card = Card::new(project_id, "Test Card", "A test card");
        db.create_card(&card).unwrap();

        // Read
        let fetched = db.get_card(card.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test Card");

        // List
        let cards = db.list_cards(project_id).unwrap();
        assert_eq!(cards.len(), 1);

        // Update
        let mut updated = fetched;
        updated.name = "Updated Card".to_string();
        db.update_card(&updated).unwrap();

        let fetched = db.get_card(card.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Card");

        // Delete
        db.delete_card(card.id).unwrap();
        let fetched = db.get_card(card.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_list_enabled_cards() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let mut card1 = Card::new(project_id, "Enabled", "");
        card1.enabled = true;
        db.create_card(&card1).unwrap();

        let mut card2 = Card::new(project_id, "Disabled", "");
        card2.enabled = false;
        db.create_card(&card2).unwrap();

        let enabled = db.list_enabled_cards(project_id).unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "Enabled");
    }
}
