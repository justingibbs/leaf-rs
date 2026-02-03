//! File system watcher implementation

use notify::{
    event::{CreateKind, ModifyKind},
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::debouncer::Debouncer;
use crate::error::{WatcherError, WatcherResult};
use crate::matcher::PatternMatcher;

/// Configuration for watching a path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// The path to watch
    pub path: PathBuf,
    /// Glob patterns to match (empty = match all)
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Debounce time in milliseconds
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
}

fn default_debounce() -> u64 {
    500
}

impl WatchConfig {
    /// Create a new watch config for a path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            patterns: Vec::new(),
            debounce_ms: default_debounce(),
        }
    }

    /// Add a pattern to match
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Set the debounce duration
    pub fn with_debounce(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }
}

/// The kind of file event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchEventKind {
    /// A new file was created
    Created,
    /// An existing file was modified
    Modified,
}

/// A file event detected by the watcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// Unique ID for this event
    pub id: Uuid,
    /// The kind of event
    pub kind: WatchEventKind,
    /// The file path
    pub path: PathBuf,
    /// The watched root path this event came from
    pub watch_path: PathBuf,
    /// Timestamp when the event was detected
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WatchEvent {
    /// Create a new watch event
    pub fn new(kind: WatchEventKind, path: PathBuf, watch_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            path,
            watch_path,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Internal state for a watched path
struct WatchedPath {
    #[allow(dead_code)]
    config: WatchConfig,
    matcher: PatternMatcher,
    debouncer: Debouncer,
}

/// File system watcher that monitors folders for file changes
///
/// Uses the `notify` crate for cross-platform file watching with
/// pattern matching and debouncing support.
pub struct FileWatcher {
    /// The underlying notify watcher
    watcher: RecommendedWatcher,
    /// Channel for sending events (kept for potential future use)
    #[allow(dead_code)]
    event_tx: mpsc::Sender<WatchEvent>,
    /// Tracked watch paths with their configs
    watched_paths: Arc<Mutex<HashMap<PathBuf, WatchedPath>>>,
}

impl FileWatcher {
    /// Create a new file watcher that sends events to the given channel
    pub fn new(event_tx: mpsc::Sender<WatchEvent>) -> WatcherResult<Self> {
        let watched_paths: Arc<Mutex<HashMap<PathBuf, WatchedPath>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let watched_paths_clone = Arc::clone(&watched_paths);
        let event_tx_clone = event_tx.clone();

        let watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    Self::handle_event(event, &watched_paths_clone, &event_tx_clone);
                }
                Err(e) => {
                    error!("Watch error: {:?}", e);
                }
            }
        })
        .map_err(|e| WatcherError::WatcherCreation(e.to_string()))?;

        info!("File watcher created");

        Ok(Self {
            watcher,
            event_tx,
            watched_paths,
        })
    }

    /// Start watching a path with the given configuration
    pub fn watch(&mut self, config: WatchConfig) -> WatcherResult<()> {
        let path = config.path.clone();

        // Validate the path exists and is a directory
        if !path.exists() {
            return Err(WatcherError::PathNotFound(path));
        }
        if !path.is_dir() {
            return Err(WatcherError::NotADirectory(path));
        }

        // Create the pattern matcher
        let matcher = PatternMatcher::new(&config.patterns)?;

        // Create the debouncer
        let debouncer = Debouncer::new(config.debounce_ms);

        // Start watching with notify
        self.watcher
            .watch(&path, RecursiveMode::Recursive)
            .map_err(|e| WatcherError::WatchPath {
                path: path.clone(),
                reason: e.to_string(),
            })?;

        // Store the watched path config
        let mut paths = self.watched_paths.lock().unwrap();
        paths.insert(
            path.clone(),
            WatchedPath {
                config,
                matcher,
                debouncer,
            },
        );

        info!("Started watching: {}", path.display());
        Ok(())
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> WatcherResult<()> {
        let path = path.as_ref();

        self.watcher
            .unwatch(path)
            .map_err(|e| WatcherError::UnwatchPath {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        let mut paths = self.watched_paths.lock().unwrap();
        paths.remove(path);

        info!("Stopped watching: {}", path.display());
        Ok(())
    }

    /// Get the list of currently watched paths
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        let paths = self.watched_paths.lock().unwrap();
        paths.keys().cloned().collect()
    }

    /// Check if a path is being watched
    pub fn is_watching(&self, path: impl AsRef<Path>) -> bool {
        let paths = self.watched_paths.lock().unwrap();
        paths.contains_key(path.as_ref())
    }

    /// Handle a notify event
    fn handle_event(
        event: Event,
        watched_paths: &Arc<Mutex<HashMap<PathBuf, WatchedPath>>>,
        event_tx: &mpsc::Sender<WatchEvent>,
    ) {
        // Determine the event kind
        let kind = match event.kind {
            EventKind::Create(CreateKind::File) => Some(WatchEventKind::Created),
            EventKind::Create(CreateKind::Any) => Some(WatchEventKind::Created),
            EventKind::Modify(ModifyKind::Data(_)) => Some(WatchEventKind::Modified),
            EventKind::Modify(ModifyKind::Any) => Some(WatchEventKind::Modified),
            _ => None,
        };

        let Some(kind) = kind else {
            trace!("Ignoring event kind: {:?}", event.kind);
            return;
        };

        // Process each path in the event
        for event_path in event.paths {
            // Skip directories
            if event_path.is_dir() {
                continue;
            }

            // Find the matching watched path
            let mut paths = watched_paths.lock().unwrap();

            // Find which watch config this path belongs to
            let watch_path = paths
                .keys()
                .find(|wp| event_path.starts_with(wp))
                .cloned();

            let Some(watch_path) = watch_path else {
                trace!("Event path doesn't match any watched paths: {:?}", event_path);
                continue;
            };

            let watched = match paths.get_mut(&watch_path) {
                Some(w) => w,
                None => continue,
            };

            // Check pattern matching
            if !watched.matcher.matches(&event_path) {
                trace!(
                    "Event path doesn't match patterns: {:?} (patterns: {:?})",
                    event_path,
                    watched.matcher.patterns()
                );
                continue;
            }

            // Check debouncing
            if !watched.debouncer.should_process(&event_path) {
                trace!("Event debounced: {:?}", event_path);
                continue;
            }

            // Create and send the event
            let watch_event = WatchEvent::new(kind, event_path.clone(), watch_path);
            debug!(
                "Sending event: {:?} for {}",
                watch_event.kind,
                watch_event.path.display()
            );

            // Drop the lock before sending to avoid deadlock
            drop(paths);

            if let Err(e) = event_tx.try_send(watch_event) {
                warn!("Failed to send watch event: {:?}", e);
            }
        }
    }

    /// Periodically cleanup old debounce entries
    pub fn cleanup_debouncers(&mut self) {
        let mut paths = self.watched_paths.lock().unwrap();
        for watched in paths.values_mut() {
            watched.debouncer.cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let watcher = FileWatcher::new(tx);
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_watch_nonexistent_path() {
        let (tx, _rx) = mpsc::channel(100);
        let mut watcher = FileWatcher::new(tx).unwrap();

        let result = watcher.watch(WatchConfig::new("/nonexistent/path"));
        assert!(matches!(result, Err(WatcherError::PathNotFound(_))));
    }

    #[tokio::test]
    async fn test_watch_file_not_directory() {
        let (tx, _rx) = mpsc::channel(100);
        let mut watcher = FileWatcher::new(tx).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let result = watcher.watch(WatchConfig::new(&file_path));
        assert!(matches!(result, Err(WatcherError::NotADirectory(_))));
    }

    #[tokio::test]
    async fn test_watch_and_unwatch() {
        let (tx, _rx) = mpsc::channel(100);
        let mut watcher = FileWatcher::new(tx).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Watch
        watcher.watch(WatchConfig::new(&path)).unwrap();
        assert!(watcher.is_watching(&path));

        // Unwatch
        watcher.unwatch(&path).unwrap();
        assert!(!watcher.is_watching(&path));
    }

    /// Integration test for file creation detection.
    /// This test is ignored by default as it depends on OS file system notifications
    /// which can be timing-sensitive. Run with `cargo test -- --ignored` to include.
    #[tokio::test]
    #[ignore = "Integration test - depends on OS file system notifications"]
    async fn test_detect_file_creation() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = FileWatcher::new(tx).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        watcher
            .watch(WatchConfig::new(&path).with_debounce(50))
            .unwrap();

        // Give the watcher time to set up
        sleep(Duration::from_millis(100)).await;

        // Create a file
        let file_path = path.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();
        file.sync_all().unwrap();

        // Wait for the event with generous timeout
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.kind, WatchEventKind::Created);
        assert_eq!(event.path, file_path);
    }

    /// Integration test for pattern filtering.
    /// This test is ignored by default as it depends on OS file system notifications.
    #[tokio::test]
    #[ignore = "Integration test - depends on OS file system notifications"]
    async fn test_pattern_filtering() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = FileWatcher::new(tx).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        watcher
            .watch(
                WatchConfig::new(&path)
                    .with_pattern("*.txt")
                    .with_debounce(50),
            )
            .unwrap();

        // Give the watcher time to set up
        sleep(Duration::from_millis(100)).await;

        // Create a .txt file (should match)
        let txt_file = path.join("test.txt");
        fs::write(&txt_file, "hello").unwrap();

        // Create a .pdf file (should NOT match)
        let pdf_file = path.join("test.pdf");
        fs::write(&pdf_file, "hello").unwrap();

        // Wait for events with generous timeout
        sleep(Duration::from_millis(500)).await;

        // Should receive only the .txt event
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        assert!(!events.is_empty(), "Should receive at least one event");
        for event in &events {
            assert!(
                event.path.extension().map(|e| e == "txt").unwrap_or(false),
                "Should only receive .txt events, got: {:?}",
                event.path
            );
        }
    }
}
