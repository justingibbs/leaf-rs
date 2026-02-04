//! Filesystem tools for the LEAF agent
//!
//! These tools allow the agent to read and write files within the project directory.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tracing::{debug, warn};

use super::{Tool, ToolContext};
use crate::error::{AgentError, AgentResult};
use crate::json_schema;

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
            .map_err(|e| AgentError::IoError(e))?;

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
        "Write content to a file in the project directory. Cannot write to .leaf/ directory. The path should be relative to the project root."
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
        if args.path.starts_with(".leaf/") || args.path.starts_with(".leaf\\") {
            return Err(AgentError::ToolError(
                "Cannot write to .leaf/ directory - use card tools to create cards".to_string(),
            ));
        }

        // Validate and resolve path
        let full_path = validate_path(&ctx.project_path, &args.path)?;

        debug!("Writing file: {}", full_path.display());

        // Create parent directories if requested
        if args.create_dirs {
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AgentError::IoError(e))?;
            }
        }

        // Write the file
        tokio::fs::write(&full_path, &args.content)
            .await
            .map_err(|e| AgentError::IoError(e))?;

        Ok(serde_json::json!({
            "path": args.path,
            "bytes_written": args.content.len(),
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

/// Validate that a path is within the project directory and resolve it
fn validate_path(project_path: &PathBuf, relative_path: &str) -> AgentResult<PathBuf> {
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
        full_path.canonicalize().map_err(|e| AgentError::IoError(e))?
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
                .map_err(|e| AgentError::IoError(e))?
                .join(full_path.file_name().unwrap_or_default())
        }
    };

    // Verify the path is within the project
    let project_canonical = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.clone());

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
    dir_path: &PathBuf,
    project_root: &PathBuf,
    recursive: bool,
    include_hidden: bool,
) -> AgentResult<Vec<FileEntry>> {
    let mut entries = Vec::new();

    let mut read_dir = tokio::fs::read_dir(dir_path)
        .await
        .map_err(|e| AgentError::IoError(e))?;

    while let Some(entry) = read_dir.next_entry().await.map_err(|e| AgentError::IoError(e))? {
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

        let metadata = entry.metadata().await.map_err(|e| AgentError::IoError(e))?;
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
