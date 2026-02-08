//! Filesystem tools for the LEAF agent
//!
//! These tools allow the agent to read, write, and manage files within the project directory.

use async_trait::async_trait;
use leaf_core::{Artifact, ArtifactType, LeafEvent};
use leaf_db::ArtifactQueries;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tracing::{debug, info, warn};

use super::{Tool, ToolContext};
use crate::error::{AgentError, AgentResult};
use crate::json_schema;

/// Register a file artifact created by the agent
fn register_artifact(ctx: &ToolContext, relative_path: &str, full_path: &PathBuf) {
    let filename = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(relative_path)
        .to_string();

    let mut artifact = Artifact::new(
        ctx.project_id,
        relative_path,
        &filename,
        ArtifactType::File,
        "agent",
    );

    // Try to get file size
    if let Ok(metadata) = std::fs::metadata(full_path) {
        artifact.size_bytes = Some(metadata.len() as i64);
    }

    match ctx.db.create_artifact(&artifact) {
        Ok(_) => {
            debug!("Registered artifact: {}", relative_path);
            if let Err(e) = ctx
                .app_handle
                .emit("leaf-event", &LeafEvent::ArtifactCreated(artifact))
            {
                warn!("Failed to emit ArtifactCreated event: {}", e);
            }
        }
        Err(e) => {
            // Artifact may already exist (e.g., overwrite); try updating instead
            debug!(
                "Could not create artifact (may already exist): {}. Updating modified.",
                e
            );
            if let Err(e2) = ctx
                .db
                .update_artifact_modified(ctx.project_id, relative_path)
            {
                warn!("Failed to update artifact modified: {}", e2);
            }
        }
    }
}

/// Register a directory artifact created by the agent
fn register_directory_artifact(ctx: &ToolContext, relative_path: &str) {
    let dirname = std::path::Path::new(relative_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(relative_path)
        .to_string();

    let artifact = Artifact::new(
        ctx.project_id,
        relative_path,
        &dirname,
        ArtifactType::Directory,
        "agent",
    );

    match ctx.db.create_artifact(&artifact) {
        Ok(_) => {
            debug!("Registered directory artifact: {}", relative_path);
            if let Err(e) = ctx
                .app_handle
                .emit("leaf-event", &LeafEvent::ArtifactCreated(artifact))
            {
                warn!("Failed to emit ArtifactCreated event: {}", e);
            }
        }
        Err(e) => {
            debug!("Could not create directory artifact (may already exist): {}", e);
        }
    }
}

/// Tool for reading files from the project
pub struct ReadFileTool;

#[derive(Debug, Deserialize)]
struct ReadFileArgs {
    path: String,
    #[serde(default)]
    max_lines: Option<usize>,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from the project directory. The path should be relative to the project root."
    }

    fn parameters_schema(&self) -> Value {
        json_schema!(
            description: "Read a file from the project",
            properties: {
                "path" => {
                    type: "string",
                    description: "Relative path to the file within the project directory",
                    required: true
                },
                "max_lines" => {
                    type: "integer",
                    description: "Maximum number of lines to read (optional, defaults to all)"
                }
            }
        )
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let args: ReadFileArgs = serde_json::from_value(args)?;

        // Validate and resolve path
        let full_path = validate_path(&ctx.project_path, &args.path)?;

        debug!("Reading file: {}", full_path.display());

        // Read the file
        let content = tokio::fs::read_to_string(&full_path)
            .await
            .map_err(AgentError::IoError)?;

        // Apply line limit if specified
        let content = if let Some(max_lines) = args.max_lines {
            content
                .lines()
                .take(max_lines)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            content
        };

        Ok(serde_json::json!({
            "path": args.path,
            "content": content,
            "size": content.len()
        }))
    }
}

/// Tool for writing files to the project
pub struct WriteFileTool;

#[derive(Debug, Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
    #[serde(default)]
    create_dirs: bool,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file in the project directory. Cannot write to .leaf/ directory. The path should be relative to the project root. The file is automatically registered as an artifact."
    }

    fn parameters_schema(&self) -> Value {
        json_schema!(
            description: "Write content to a file",
            properties: {
                "path" => {
                    type: "string",
                    description: "Relative path to the file within the project directory (not .leaf/)",
                    required: true
                },
                "content" => {
                    type: "string",
                    description: "Content to write to the file",
                    required: true
                },
                "create_dirs" => {
                    type: "boolean",
                    description: "Create parent directories if they don't exist (default: false)"
                }
            }
        )
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let args: WriteFileArgs = serde_json::from_value(args)?;

        // Validate path - don't allow writing to .leaf/
        validate_not_leaf_dir(&args.path)?;

        // Validate and resolve path
        let full_path = validate_path(&ctx.project_path, &args.path)?;

        debug!("Writing file: {}", full_path.display());

        // Create parent directories if requested
        if args.create_dirs {
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(AgentError::IoError)?;
            }
        }

        // Write the file
        tokio::fs::write(&full_path, &args.content)
            .await
            .map_err(AgentError::IoError)?;

        // Register as artifact
        register_artifact(ctx, &args.path, &full_path);

        Ok(serde_json::json!({
            "path": args.path,
            "bytes_written": args.content.len(),
            "success": true
        }))
    }
}

/// Tool for creating folders in the project
pub struct CreateFolderTool;

#[async_trait]
impl Tool for CreateFolderTool {
    fn name(&self) -> &str {
        "create_folder"
    }

    fn description(&self) -> &str {
        "Create a folder in the project directory. Creates parent directories as needed. Cannot create folders inside .leaf/."
    }

    fn parameters_schema(&self) -> Value {
        json_schema!(
            description: "Create a folder in the project",
            properties: {
                "path" => {
                    type: "string",
                    description: "Folder path relative to project root (e.g., 'inbox', 'output/reports')",
                    required: true
                }
            }
        )
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::InvalidToolCall("path is required".to_string()))?;

        validate_not_leaf_dir(path)?;

        let full_path = validate_path(&ctx.project_path, path)?;

        debug!("Creating folder: {}", full_path.display());

        tokio::fs::create_dir_all(&full_path)
            .await
            .map_err(AgentError::IoError)?;

        info!("Created folder: {}", path);

        // Register as directory artifact
        register_directory_artifact(ctx, path);

        Ok(serde_json::json!({
            "path": path,
            "success": true
        }))
    }
}

/// Tool for deleting files from the project
pub struct DeleteFileTool;

#[async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file from the project directory. Cannot delete files inside .leaf/."
    }

    fn parameters_schema(&self) -> Value {
        json_schema!(
            description: "Delete a file from the project",
            properties: {
                "path" => {
                    type: "string",
                    description: "File path relative to project root",
                    required: true
                }
            }
        )
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::InvalidToolCall("path is required".to_string()))?;

        validate_not_leaf_dir(path)?;

        let full_path = validate_path(&ctx.project_path, path)?;

        if !full_path.exists() {
            return Err(AgentError::ToolError(format!("File not found: {}", path)));
        }

        if full_path.is_dir() {
            return Err(AgentError::ToolError(
                "Cannot delete a directory with delete_file. Use it only for files.".to_string(),
            ));
        }

        debug!("Deleting file: {}", full_path.display());

        tokio::fs::remove_file(&full_path)
            .await
            .map_err(AgentError::IoError)?;

        info!("Deleted file: {}", path);

        // Mark artifact as deleted
        if let Err(e) = ctx.db.mark_artifact_deleted(ctx.project_id, path) {
            debug!("Could not mark artifact deleted (may not exist): {}", e);
        }

        Ok(serde_json::json!({
            "path": path,
            "success": true
        }))
    }
}

/// Tool for moving/renaming files in the project
pub struct MoveFileTool;

#[async_trait]
impl Tool for MoveFileTool {
    fn name(&self) -> &str {
        "move_file"
    }

    fn description(&self) -> &str {
        "Move or rename a file within the project directory. Cannot move files into or out of .leaf/."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Move or rename a file",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Current path relative to project root"
                },
                "destination": {
                    "type": "string",
                    "description": "New path relative to project root"
                }
            },
            "required": ["source", "destination"]
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let source = args
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::InvalidToolCall("source is required".to_string()))?;
        let destination = args
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::InvalidToolCall("destination is required".to_string()))?;

        validate_not_leaf_dir(source)?;
        validate_not_leaf_dir(destination)?;

        let source_path = validate_path(&ctx.project_path, source)?;
        let dest_path = validate_path(&ctx.project_path, destination)?;

        if !source_path.exists() {
            return Err(AgentError::ToolError(format!(
                "Source file not found: {}",
                source
            )));
        }

        // Create parent directories for destination if needed
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AgentError::IoError)?;
        }

        debug!("Moving {} -> {}", source_path.display(), dest_path.display());

        tokio::fs::rename(&source_path, &dest_path)
            .await
            .map_err(AgentError::IoError)?;

        info!("Moved {} -> {}", source, destination);

        // Update artifact tracking: mark old as deleted, register new
        if let Err(e) = ctx.db.mark_artifact_deleted(ctx.project_id, source) {
            debug!("Could not mark source artifact deleted: {}", e);
        }
        register_artifact(ctx, destination, &dest_path);

        Ok(serde_json::json!({
            "source": source,
            "destination": destination,
            "success": true
        }))
    }
}

/// Tool for listing directory contents
pub struct ListDirectoryTool;

#[derive(Debug, Deserialize)]
struct ListDirectoryArgs {
    path: Option<String>,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    include_hidden: bool,
}

#[derive(Debug, Serialize)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
}

#[async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List files and directories in the project. The path should be relative to the project root, or omit for the root directory."
    }

    fn parameters_schema(&self) -> Value {
        json_schema!(
            description: "List directory contents",
            properties: {
                "path" => {
                    type: "string",
                    description: "Relative path to list (optional, defaults to project root)"
                },
                "recursive" => {
                    type: "boolean",
                    description: "List contents recursively (default: false)"
                },
                "include_hidden" => {
                    type: "boolean",
                    description: "Include hidden files starting with . (default: false)"
                }
            }
        )
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> AgentResult<Value> {
        let args: ListDirectoryArgs = serde_json::from_value(args)?;

        let dir_path = if let Some(ref path) = args.path {
            validate_path(&ctx.project_path, path)?
        } else {
            ctx.project_path.clone()
        };

        debug!("Listing directory: {}", dir_path.display());

        let entries = list_directory_entries(
            &dir_path,
            &ctx.project_path,
            args.recursive,
            args.include_hidden,
        )
        .await?;

        Ok(serde_json::json!({
            "path": args.path.unwrap_or_else(|| ".".to_string()),
            "entries": entries
        }))
    }
}

/// Validate that a path doesn't point into .leaf/
fn validate_not_leaf_dir(path: &str) -> AgentResult<()> {
    if path.starts_with(".leaf/") || path.starts_with(".leaf\\") || path == ".leaf" {
        return Err(AgentError::ToolError(
            "Cannot modify .leaf/ directory - use stack/card tools instead".to_string(),
        ));
    }
    Ok(())
}

/// Validate that a path is within the project directory and resolve it
fn validate_path(project_path: &Path, relative_path: &str) -> AgentResult<PathBuf> {
    // Normalize the path
    let normalized = relative_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();

    // Build the full path
    let full_path = project_path.join(&normalized);

    // Canonicalize to resolve any .. or symlinks
    // Note: The file might not exist yet for write operations, so we canonicalize the parent
    let canonical = if full_path.exists() {
        full_path.canonicalize().map_err(AgentError::IoError)?
    } else {
        // For new files, check the parent directory
        let parent = full_path.parent().ok_or_else(|| {
            AgentError::ToolError("Invalid path: no parent directory".to_string())
        })?;

        if !parent.exists() {
            // Parent doesn't exist yet - this is okay for write_file with create_dirs
            full_path.clone()
        } else {
            parent
                .canonicalize()
                .map_err(AgentError::IoError)?
                .join(full_path.file_name().unwrap_or_default())
        }
    };

    // Verify the path is within the project
    let project_canonical = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    if !canonical.starts_with(&project_canonical) {
        warn!(
            "Path traversal attempt: {} is outside {}",
            canonical.display(),
            project_canonical.display()
        );
        return Err(AgentError::ToolError(
            "Path must be within the project directory".to_string(),
        ));
    }

    Ok(canonical)
}

/// List directory entries recursively or non-recursively
async fn list_directory_entries(
    dir_path: &Path,
    project_root: &Path,
    recursive: bool,
    include_hidden: bool,
) -> AgentResult<Vec<FileEntry>> {
    let mut entries = Vec::new();

    let mut read_dir = tokio::fs::read_dir(dir_path)
        .await
        .map_err(AgentError::IoError)?;

    while let Some(entry) = read_dir.next_entry().await.map_err(AgentError::IoError)? {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files unless requested
        if !include_hidden && name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let metadata = entry.metadata().await.map_err(AgentError::IoError)?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };

        entries.push(FileEntry {
            name,
            path: relative_path.clone(),
            is_dir,
            size,
        });

        // Recurse into subdirectories
        if recursive && is_dir {
            let sub_entries =
                Box::pin(list_directory_entries(&path, project_root, true, include_hidden))
                    .await?;
            entries.extend(sub_entries);
        }
    }

    // Sort by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_normal() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create a test file
        std::fs::write(project_path.join("test.txt"), "content").unwrap();

        let result = validate_path(&project_path, "test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let result = validate_path(&project_path, "../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create a subdirectory with a file
        std::fs::create_dir_all(project_path.join("subdir")).unwrap();
        std::fs::write(project_path.join("subdir/file.txt"), "content").unwrap();

        let result = validate_path(&project_path, "subdir/file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_not_leaf_dir() {
        assert!(validate_not_leaf_dir("inbox/file.txt").is_ok());
        assert!(validate_not_leaf_dir("output/result.csv").is_ok());
        assert!(validate_not_leaf_dir(".leaf/programs/test").is_err());
        assert!(validate_not_leaf_dir(".leaf").is_err());
    }

    #[tokio::test]
    async fn test_list_directory_entries() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create some test files
        std::fs::write(project_path.join("file1.txt"), "content1").unwrap();
        std::fs::write(project_path.join("file2.txt"), "content2").unwrap();
        std::fs::write(project_path.join(".hidden"), "hidden").unwrap();
        std::fs::create_dir_all(project_path.join("subdir")).unwrap();

        // Non-recursive, no hidden
        let entries = list_directory_entries(&project_path, &project_path, false, false)
            .await
            .unwrap();

        assert_eq!(entries.len(), 3); // file1.txt, file2.txt, subdir
        assert!(entries.iter().all(|e| !e.name.starts_with('.')));

        // With hidden
        let entries = list_directory_entries(&project_path, &project_path, false, true)
            .await
            .unwrap();

        assert_eq!(entries.len(), 4); // includes .hidden
    }
}
