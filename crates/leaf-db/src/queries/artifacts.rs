//! Artifact database operations (stubs for Phase A)

use leaf_core::{Artifact, Result};
use uuid::Uuid;

use crate::Database;

/// Extension trait for artifact operations
pub trait ArtifactQueries {
    fn create_artifact(&self, artifact: &Artifact) -> Result<()>;
    fn get_artifact(&self, id: Uuid) -> Result<Option<Artifact>>;
    fn list_artifacts_for_project(&self, project_id: Uuid) -> Result<Vec<Artifact>>;
    fn delete_artifact(&self, id: Uuid) -> Result<()>;
}

impl ArtifactQueries for Database {
    fn create_artifact(&self, _artifact: &Artifact) -> Result<()> {
        // TODO: Implement in Phase D
        Ok(())
    }

    fn get_artifact(&self, _id: Uuid) -> Result<Option<Artifact>> {
        // TODO: Implement in Phase D
        Ok(None)
    }

    fn list_artifacts_for_project(&self, _project_id: Uuid) -> Result<Vec<Artifact>> {
        // TODO: Implement in Phase D
        Ok(Vec::new())
    }

    fn delete_artifact(&self, _id: Uuid) -> Result<()> {
        // TODO: Implement in Phase D
        Ok(())
    }
}
