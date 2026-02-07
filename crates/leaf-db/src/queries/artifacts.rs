//! Artifact database operations

use leaf_core::{Artifact, ArtifactStatus, ArtifactType, LeafError, Result};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for artifact operations
pub trait ArtifactQueries {
    fn create_artifact(&self, artifact: &Artifact) -> Result<()>;
    fn get_artifact(&self, id: Uuid) -> Result<Option<Artifact>>;
    fn get_artifact_by_path(&self, project_id: Uuid, path: &str) -> Result<Option<Artifact>>;
    fn list_artifacts_for_project(&self, project_id: Uuid) -> Result<Vec<Artifact>>;
    fn list_artifacts_by_status(
        &self,
        project_id: Uuid,
        status: &ArtifactStatus,
    ) -> Result<Vec<Artifact>>;
    fn update_artifact_modified(&self, project_id: Uuid, path: &str) -> Result<()>;
    fn mark_artifact_deleted(&self, project_id: Uuid, path: &str) -> Result<()>;
    fn delete_artifact(&self, id: Uuid) -> Result<()>;
}

impl ArtifactQueries for Database {
    fn create_artifact(&self, artifact: &Artifact) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO artifacts (id, project_id, path, filename, artifact_type, mime_type, size_bytes, created_by, created_by_execution_id, status, created_at, modified_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                artifact.id.to_string(),
                artifact.project_id.to_string(),
                artifact.path,
                artifact.filename,
                serde_json::to_string(&artifact.artifact_type)?,
                artifact.mime_type,
                artifact.size_bytes,
                artifact.created_by,
                artifact.created_by_execution_id.map(|id| id.to_string()),
                serde_json::to_string(&artifact.status)?,
                artifact.created_at.to_rfc3339(),
                artifact.modified_at.to_rfc3339(),
                artifact.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()),
            ],
        )?;
        Ok(())
    }

    fn get_artifact(&self, id: Uuid) -> Result<Option<Artifact>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, path, filename, artifact_type, mime_type, size_bytes, created_by, created_by_execution_id, status, created_at, modified_at, metadata
             FROM artifacts WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_artifact);

        match result {
            Ok(artifact) => Ok(Some(artifact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn get_artifact_by_path(&self, project_id: Uuid, path: &str) -> Result<Option<Artifact>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, path, filename, artifact_type, mime_type, size_bytes, created_by, created_by_execution_id, status, created_at, modified_at, metadata
             FROM artifacts WHERE project_id = ?1 AND path = ?2",
        )?;

        let result = stmt.query_row(
            params![project_id.to_string(), path],
            row_to_artifact,
        );

        match result {
            Ok(artifact) => Ok(Some(artifact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_artifacts_for_project(&self, project_id: Uuid) -> Result<Vec<Artifact>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, path, filename, artifact_type, mime_type, size_bytes, created_by, created_by_execution_id, status, created_at, modified_at, metadata
             FROM artifacts WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;

        let artifacts = stmt
            .query_map(params![project_id.to_string()], row_to_artifact)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(artifacts)
    }

    fn list_artifacts_by_status(
        &self,
        project_id: Uuid,
        status: &ArtifactStatus,
    ) -> Result<Vec<Artifact>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, path, filename, artifact_type, mime_type, size_bytes, created_by, created_by_execution_id, status, created_at, modified_at, metadata
             FROM artifacts WHERE project_id = ?1 AND status = ?2 ORDER BY created_at DESC",
        )?;

        let artifacts = stmt
            .query_map(
                params![
                    project_id.to_string(),
                    serde_json::to_string(status)?,
                ],
                row_to_artifact,
            )?
            .filter_map(|r| r.ok())
            .collect();

        Ok(artifacts)
    }

    fn update_artifact_modified(&self, project_id: Uuid, path: &str) -> Result<()> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        let status = serde_json::to_string(&ArtifactStatus::Modified)?;
        conn.execute(
            "UPDATE artifacts SET status = ?3, modified_at = ?4 WHERE project_id = ?1 AND path = ?2",
            params![project_id.to_string(), path, status, now],
        )?;
        Ok(())
    }

    fn mark_artifact_deleted(&self, project_id: Uuid, path: &str) -> Result<()> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        let status = serde_json::to_string(&ArtifactStatus::Deleted)?;
        conn.execute(
            "UPDATE artifacts SET status = ?3, modified_at = ?4 WHERE project_id = ?1 AND path = ?2",
            params![project_id.to_string(), path, status, now],
        )?;
        Ok(())
    }

    fn delete_artifact(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM artifacts WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}

fn row_to_artifact(row: &Row) -> rusqlite::Result<Artifact> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let artifact_type: String = row.get(4)?;
    let created_by_execution_id: Option<String> = row.get(8)?;
    let status: String = row.get(9)?;
    let created_at: String = row.get(10)?;
    let modified_at: String = row.get(11)?;
    let metadata: Option<String> = row.get(12)?;

    Ok(Artifact {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        project_id: Uuid::parse_str(&project_id).unwrap_or_default(),
        path: row.get(2)?,
        filename: row.get(3)?,
        artifact_type: serde_json::from_str(&artifact_type).unwrap_or(ArtifactType::File),
        mime_type: row.get(5)?,
        size_bytes: row.get(6)?,
        created_by: row.get(7)?,
        created_by_execution_id: created_by_execution_id
            .and_then(|s| Uuid::parse_str(&s).ok()),
        status: serde_json::from_str(&status).unwrap_or_default(),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        modified_at: chrono::DateTime::parse_from_rfc3339(&modified_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use leaf_core::ArtifactType;

    #[test]
    fn test_artifact_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create
        let mut artifact = Artifact::new(
            project_id,
            "output/report.csv",
            "report.csv",
            ArtifactType::File,
            "user",
        );
        artifact.mime_type = Some("text/csv".to_string());
        artifact.size_bytes = Some(1024);
        db.create_artifact(&artifact).unwrap();

        // Get by ID
        let fetched = db.get_artifact(artifact.id).unwrap().unwrap();
        assert_eq!(fetched.path, "output/report.csv");
        assert_eq!(fetched.filename, "report.csv");
        assert_eq!(fetched.created_by, "user");
        assert_eq!(fetched.status, ArtifactStatus::Active);

        // Get by path
        let fetched = db
            .get_artifact_by_path(project_id, "output/report.csv")
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, artifact.id);

        // List
        let artifacts = db.list_artifacts_for_project(project_id).unwrap();
        assert_eq!(artifacts.len(), 1);

        // Update modified
        db.update_artifact_modified(project_id, "output/report.csv")
            .unwrap();
        let fetched = db.get_artifact(artifact.id).unwrap().unwrap();
        assert_eq!(fetched.status, ArtifactStatus::Modified);

        // Mark deleted
        db.mark_artifact_deleted(project_id, "output/report.csv")
            .unwrap();
        let fetched = db.get_artifact(artifact.id).unwrap().unwrap();
        assert_eq!(fetched.status, ArtifactStatus::Deleted);

        // List by status
        let deleted = db
            .list_artifacts_by_status(project_id, &ArtifactStatus::Deleted)
            .unwrap();
        assert_eq!(deleted.len(), 1);

        // Delete
        db.delete_artifact(artifact.id).unwrap();
        let fetched = db.get_artifact(artifact.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_artifact_with_execution_id() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();
        let exec_id = Uuid::new_v4();
        let card_id = Uuid::new_v4();

        let mut artifact = Artifact::new(
            project_id,
            "output/result.json",
            "result.json",
            ArtifactType::File,
            format!("card:{}", card_id),
        );
        artifact.created_by_execution_id = Some(exec_id);
        db.create_artifact(&artifact).unwrap();

        let fetched = db.get_artifact(artifact.id).unwrap().unwrap();
        assert_eq!(fetched.created_by_execution_id, Some(exec_id));
        assert!(fetched.created_by.starts_with("card:"));
    }

    #[test]
    fn test_artifact_not_found() {
        let db = Database::in_memory().unwrap();
        let result = db.get_artifact(Uuid::new_v4()).unwrap();
        assert!(result.is_none());

        let result = db
            .get_artifact_by_path(Uuid::new_v4(), "nonexistent.txt")
            .unwrap();
        assert!(result.is_none());
    }
}
