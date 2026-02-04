//! MCP server database operations

use leaf_core::{LeafError, McpServer, Result};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::Database;

/// Extension trait for MCP server operations
pub trait McpServerQueries {
    fn create_mcp_server(&self, server: &McpServer) -> Result<()>;
    fn get_mcp_server(&self, id: Uuid) -> Result<Option<McpServer>>;
    fn list_mcp_servers(&self, project_id: Uuid) -> Result<Vec<McpServer>>;
    fn list_enabled_mcp_servers(&self, project_id: Uuid) -> Result<Vec<McpServer>>;
    fn update_mcp_server(&self, server: &McpServer) -> Result<()>;
    fn delete_mcp_server(&self, id: Uuid) -> Result<()>;
    fn enable_mcp_server(&self, id: Uuid) -> Result<()>;
    fn disable_mcp_server(&self, id: Uuid) -> Result<()>;
}

impl McpServerQueries for Database {
    fn create_mcp_server(&self, server: &McpServer) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO mcp_servers (id, project_id, name, command, args, env, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                server.id.to_string(),
                server.project_id.to_string(),
                server.name,
                server.command,
                serde_json::to_string(&server.args)?,
                serde_json::to_string(&server.env)?,
                server.enabled,
                server.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_mcp_server(&self, id: Uuid) -> Result<Option<McpServer>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, command, args, env, enabled, created_at
             FROM mcp_servers WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], row_to_mcp_server);

        match result {
            Ok(server) => Ok(Some(server)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LeafError::from(e)),
        }
    }

    fn list_mcp_servers(&self, project_id: Uuid) -> Result<Vec<McpServer>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, command, args, env, enabled, created_at
             FROM mcp_servers WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;

        let servers = stmt
            .query_map(params![project_id.to_string()], row_to_mcp_server)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(servers)
    }

    fn list_enabled_mcp_servers(&self, project_id: Uuid) -> Result<Vec<McpServer>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, command, args, env, enabled, created_at
             FROM mcp_servers WHERE project_id = ?1 AND enabled = 1 ORDER BY created_at DESC",
        )?;

        let servers = stmt
            .query_map(params![project_id.to_string()], row_to_mcp_server)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(servers)
    }

    fn update_mcp_server(&self, server: &McpServer) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE mcp_servers SET name = ?2, command = ?3, args = ?4, env = ?5, enabled = ?6
             WHERE id = ?1",
            params![
                server.id.to_string(),
                server.name,
                server.command,
                serde_json::to_string(&server.args)?,
                serde_json::to_string(&server.env)?,
                server.enabled,
            ],
        )?;
        Ok(())
    }

    fn delete_mcp_server(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM mcp_servers WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    fn enable_mcp_server(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE mcp_servers SET enabled = 1 WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    fn disable_mcp_server(&self, id: Uuid) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE mcp_servers SET enabled = 0 WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}

fn row_to_mcp_server(row: &Row) -> rusqlite::Result<McpServer> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let args: String = row.get(4)?;
    let env: String = row.get(5)?;
    let created_at: String = row.get(7)?;

    Ok(McpServer {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        project_id: Uuid::parse_str(&project_id).unwrap_or_default(),
        name: row.get(2)?,
        command: row.get(3)?,
        args: serde_json::from_str(&args).unwrap_or_default(),
        env: serde_json::from_str(&env).unwrap_or_default(),
        enabled: row.get(6)?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_crud() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        // Create
        let mut server = McpServer::new(project_id, "test-server", "echo");
        server.args = vec!["hello".to_string()];
        server.env.insert("FOO".to_string(), "bar".to_string());
        db.create_mcp_server(&server).unwrap();

        // Read
        let fetched = db.get_mcp_server(server.id).unwrap().unwrap();
        assert_eq!(fetched.name, "test-server");
        assert_eq!(fetched.command, "echo");
        assert_eq!(fetched.args, vec!["hello"]);
        assert_eq!(fetched.env.get("FOO"), Some(&"bar".to_string()));
        assert!(fetched.enabled);

        // List
        let servers = db.list_mcp_servers(project_id).unwrap();
        assert_eq!(servers.len(), 1);

        // Update
        let mut updated = fetched;
        updated.name = "updated-server".to_string();
        db.update_mcp_server(&updated).unwrap();

        let fetched = db.get_mcp_server(server.id).unwrap().unwrap();
        assert_eq!(fetched.name, "updated-server");

        // Delete
        db.delete_mcp_server(server.id).unwrap();
        let fetched = db.get_mcp_server(server.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_list_enabled_mcp_servers() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let mut server1 = McpServer::new(project_id, "enabled", "echo");
        server1.enabled = true;
        db.create_mcp_server(&server1).unwrap();

        let mut server2 = McpServer::new(project_id, "disabled", "echo");
        server2.enabled = false;
        db.create_mcp_server(&server2).unwrap();

        let enabled = db.list_enabled_mcp_servers(project_id).unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "enabled");
    }

    #[test]
    fn test_enable_disable_mcp_server() {
        let db = Database::in_memory().unwrap();
        let project_id = Uuid::new_v4();

        let server = McpServer::new(project_id, "test", "echo");
        db.create_mcp_server(&server).unwrap();

        // Initially enabled
        let fetched = db.get_mcp_server(server.id).unwrap().unwrap();
        assert!(fetched.enabled);

        // Disable
        db.disable_mcp_server(server.id).unwrap();
        let fetched = db.get_mcp_server(server.id).unwrap().unwrap();
        assert!(!fetched.enabled);

        // Enable
        db.enable_mcp_server(server.id).unwrap();
        let fetched = db.get_mcp_server(server.id).unwrap().unwrap();
        assert!(fetched.enabled);
    }
}
