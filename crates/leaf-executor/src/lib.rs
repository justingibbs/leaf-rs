//! LEAF Executor - Sandboxed code execution for LEAF
//!
//! This crate provides sandboxed TypeScript execution using Deno.
//! Cards run in isolated environments with:
//! - Limited filesystem access (project folder only)
//! - No network by default
//! - Configurable timeouts
//! - Automatic retries
//!
//! # Example
//!
//! ```ignore
//! use leaf_executor::Executor;
//! use leaf_core::Card;
//! use std::path::Path;
//!
//! let executor = Executor::new()?;
//! let result = executor.execute(&card, None, Path::new("/project")).await?;
//! println!("Exit code: {}", result.exit_code);
//! ```

mod deno;
mod error;

pub use deno::DenoConfig;
pub use error::{ExecutorError, Result};

use leaf_core::{Card, Event, EventPayload};
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Execution result from running a card's program
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code from the process (0 = success)
    pub exit_code: i32,
    /// Captured stdout output
    pub stdout: String,
    /// Captured stderr output
    pub stderr: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Executor for running card programs in a sandboxed Deno environment
#[derive(Debug, Clone)]
pub struct Executor {
    /// Deno configuration
    deno_config: DenoConfig,
}

impl Executor {
    /// Create a new executor by discovering Deno on the system
    pub fn new() -> Result<Self> {
        let deno_config = DenoConfig::discover()?;
        info!("Executor initialized with Deno: {}", deno_config.deno_path.display());
        Ok(Self { deno_config })
    }

    /// Execute a card's program
    ///
    /// # Arguments
    /// * `card` - The card to execute
    /// * `event` - Optional triggering event (for file triggers, etc.)
    /// * `project_root` - Path to the project root directory
    ///
    /// # Returns
    /// An `ExecutionResult` with exit code, stdout, stderr, and duration
    pub async fn execute(
        &self,
        card: &Card,
        event: Option<&Event>,
        project_root: &Path,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Build program path: <project_root>/.leaf/programs/<card_id>/main.ts
        let program_dir = project_root
            .join(".leaf")
            .join("programs")
            .join(card.id.to_string());
        let program_path = program_dir.join(&card.program.entrypoint);

        // Verify program exists
        if !program_path.exists() {
            return Err(ExecutorError::ProgramNotFound(program_path));
        }

        info!(
            "Executing card '{}' program: {}",
            card.name,
            program_path.display()
        );

        // Build environment variables
        let event_payload = event
            .map(|e| serde_json::to_string(&e.payload).unwrap_or_default())
            .unwrap_or_else(|| {
                // For manual triggers, create a manual payload
                serde_json::to_string(&EventPayload::Manual { input: None }).unwrap_or_default()
            });

        // Build Deno command
        let args = self.deno_config.build_args(project_root, &program_path);

        let mut cmd = Command::new(&self.deno_config.deno_path);
        cmd.args(&args)
            .current_dir(project_root)
            .env("LEAF_EVENT_PAYLOAD", &event_payload)
            .env("LEAF_PROJECT_ROOT", project_root.to_string_lossy().as_ref())
            .env("LEAF_CARD_ID", card.id.to_string())
            .env("LEAF_EXECUTION_ID", Uuid::new_v4().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Running command: {:?}", cmd);

        // Spawn the process
        let mut child = cmd.spawn()?;

        // Capture stdout and stderr concurrently
        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();

        let stdout_task = tokio::spawn(async move {
            let mut output = String::new();
            if let Some(stdout) = stdout_handle {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    output.push_str(&line);
                    output.push('\n');
                }
            }
            output
        });

        let stderr_task = tokio::spawn(async move {
            let mut output = String::new();
            if let Some(stderr) = stderr_handle {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    output.push_str(&line);
                    output.push('\n');
                }
            }
            output
        });

        // Wait for the process with timeout
        let timeout_secs = card.program.timeout_secs;
        let timeout_duration = std::time::Duration::from_secs(timeout_secs as u64);

        let status = tokio::select! {
            result = child.wait() => {
                result?
            }
            _ = tokio::time::sleep(timeout_duration) => {
                warn!("Execution timed out after {} seconds, killing process", timeout_secs);
                // Try to kill the process
                let _ = child.kill().await;
                return Err(ExecutorError::Timeout(timeout_secs));
            }
        };

        // Collect stdout and stderr
        let stdout = stdout_task.await.unwrap_or_default();
        let stderr = stderr_task.await.unwrap_or_default();

        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = status.code().unwrap_or(-1);

        if exit_code == 0 {
            info!(
                "Execution completed successfully in {}ms",
                duration_ms
            );
        } else {
            error!(
                "Execution failed with exit code {} in {}ms",
                exit_code, duration_ms
            );
            debug!("stderr: {}", stderr);
        }

        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr,
            duration_ms,
        })
    }

    /// Get the path to the Deno executable
    pub fn deno_path(&self) -> &Path {
        &self.deno_config.deno_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation_without_deno() {
        // This test will fail if Deno is not installed, which is expected behavior
        let result = Executor::new();
        // We just check it doesn't panic - the result depends on whether Deno is installed
        match result {
            Ok(executor) => {
                println!("Deno found at: {}", executor.deno_path().display());
            }
            Err(ExecutorError::DenoNotFound) => {
                println!("Deno not installed - this is expected in some environments");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn test_execution_result_default() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: "Hello, world!".to_string(),
            stderr: String::new(),
            duration_ms: 100,
        };
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Hello, world!");
    }
}
