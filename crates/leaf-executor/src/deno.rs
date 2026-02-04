//! Deno runtime discovery and configuration

use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::error::{ExecutorError, Result};

/// Configuration for the Deno runtime
#[derive(Debug, Clone)]
pub struct DenoConfig {
    /// Path to the Deno executable
    pub deno_path: PathBuf,
}

impl DenoConfig {
    /// Discover Deno on the system
    ///
    /// Search order:
    /// 1. DENO_PATH environment variable
    /// 2. System PATH (via `which deno`)
    /// 3. Common installation paths
    pub fn discover() -> Result<Self> {
        // 1. Check DENO_PATH environment variable
        if let Ok(path) = std::env::var("DENO_PATH") {
            let deno_path = PathBuf::from(&path);
            if deno_path.exists() {
                info!("Found Deno via DENO_PATH: {}", deno_path.display());
                return Ok(Self { deno_path });
            }
            debug!("DENO_PATH set but file not found: {}", path);
        }

        // 2. Check system PATH
        if let Ok(path) = which::which("deno") {
            info!("Found Deno in PATH: {}", path.display());
            return Ok(Self { deno_path: path });
        }

        // 3. Check common installation paths
        let common_paths = Self::common_deno_paths();
        for path in common_paths {
            if path.exists() {
                info!("Found Deno at common path: {}", path.display());
                return Ok(Self { deno_path: path });
            }
        }

        Err(ExecutorError::DenoNotFound)
    }

    /// Get common Deno installation paths based on the current platform
    fn common_deno_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Get home directory
        if let Some(home) = dirs::home_dir() {
            // Default deno install location
            paths.push(home.join(".deno").join("bin").join("deno"));

            // Homebrew on macOS (Intel)
            paths.push(PathBuf::from("/usr/local/bin/deno"));

            // Homebrew on macOS (Apple Silicon)
            paths.push(PathBuf::from("/opt/homebrew/bin/deno"));

            // Cargo install location
            paths.push(home.join(".cargo").join("bin").join("deno"));
        }

        // Linux common paths
        paths.push(PathBuf::from("/usr/bin/deno"));
        paths.push(PathBuf::from("/usr/local/bin/deno"));

        paths
    }

    /// Build Deno command arguments for sandboxed execution
    ///
    /// Sets up permissions:
    /// - Read access to project directory
    /// - Write access to .leaf/outputs directory
    /// - Limited environment variables
    /// - No network access
    /// - No prompt for permissions
    pub fn build_args(&self, project_root: &Path, program_path: &Path) -> Vec<String> {
        let outputs_dir = project_root.join(".leaf").join("outputs");

        let args = vec![
            "run".to_string(),
            // Allow reading from project directory
            format!("--allow-read={}", project_root.display()),
            // Allow writing to outputs directory only
            format!("--allow-write={}", outputs_dir.display()),
            // Allow specific environment variables
            "--allow-env=LEAF_EVENT_PAYLOAD,LEAF_PROJECT_ROOT,LEAF_CARD_ID,LEAF_EXECUTION_ID"
                .to_string(),
            // No interactive permission prompts
            "--no-prompt".to_string(),
            // The program to run
            program_path.to_string_lossy().to_string(),
        ];

        debug!("Deno args: {:?}", args);
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_build_args() {
        let config = DenoConfig {
            deno_path: PathBuf::from("/usr/bin/deno"),
        };

        let project_root = Path::new("/home/user/project");
        let program_path = Path::new("/home/user/project/.leaf/programs/card-123/main.ts");

        let args = config.build_args(project_root, program_path);

        assert!(args.contains(&"run".to_string()));
        assert!(args
            .iter()
            .any(|a| a.starts_with("--allow-read=/home/user/project")));
        assert!(args
            .iter()
            .any(|a| a.starts_with("--allow-write=/home/user/project/.leaf/outputs")));
        assert!(args.contains(&"--no-prompt".to_string()));
        assert!(args
            .iter()
            .any(|a| a.contains("/home/user/project/.leaf/programs/card-123/main.ts")));
    }

    #[test]
    fn test_common_deno_paths() {
        let paths = DenoConfig::common_deno_paths();
        // Should have at least some common paths
        assert!(!paths.is_empty());
    }
}
