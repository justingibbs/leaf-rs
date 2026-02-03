//! Debouncing logic to prevent duplicate file events

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Debouncer for file events
///
/// Prevents duplicate events for the same file within a configurable time window.
/// This is useful because file system operations often trigger multiple events
/// (e.g., a file save might trigger both a modify and a create event).
#[derive(Debug)]
pub struct Debouncer {
    /// Map of file paths to their last event time
    last_events: HashMap<PathBuf, Instant>,
    /// Debounce duration
    duration: Duration,
}

impl Debouncer {
    /// Create a new debouncer with the specified duration in milliseconds
    pub fn new(duration_ms: u64) -> Self {
        Self {
            last_events: HashMap::new(),
            duration: Duration::from_millis(duration_ms),
        }
    }

    /// Check if an event for this path should be processed
    ///
    /// Returns `true` if the event should be processed (not a duplicate),
    /// `false` if the event should be ignored (too soon after the last event).
    pub fn should_process(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref().to_path_buf();
        let now = Instant::now();

        if let Some(last) = self.last_events.get(&path) {
            if now.duration_since(*last) < self.duration {
                return false;
            }
        }

        self.last_events.insert(path, now);
        true
    }

    /// Clear old entries from the debouncer
    ///
    /// Removes entries older than the debounce duration to prevent memory growth.
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.last_events
            .retain(|_, last| now.duration_since(*last) < self.duration * 2);
    }

    /// Get the debounce duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.duration.as_millis() as u64
    }

    /// Clear all tracked events
    pub fn clear(&mut self) {
        self.last_events.clear();
    }
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(500) // 500ms default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_debouncer_basic() {
        let mut debouncer = Debouncer::new(100);

        // First event should process
        assert!(debouncer.should_process("/test/file.txt"));

        // Immediate second event should be debounced
        assert!(!debouncer.should_process("/test/file.txt"));

        // Different file should process
        assert!(debouncer.should_process("/test/other.txt"));
    }

    #[test]
    fn test_debouncer_timeout() {
        let mut debouncer = Debouncer::new(50);

        assert!(debouncer.should_process("/test/file.txt"));
        assert!(!debouncer.should_process("/test/file.txt"));

        // Wait for debounce timeout
        sleep(Duration::from_millis(60));

        // Should process again after timeout
        assert!(debouncer.should_process("/test/file.txt"));
    }

    #[test]
    fn test_debouncer_cleanup() {
        let mut debouncer = Debouncer::new(50);

        debouncer.should_process("/test/file1.txt");
        debouncer.should_process("/test/file2.txt");

        sleep(Duration::from_millis(120));

        debouncer.cleanup();

        // Old entries should be removed
        assert!(debouncer.last_events.is_empty());
    }

    #[test]
    fn test_debouncer_clear() {
        let mut debouncer = Debouncer::new(100);

        debouncer.should_process("/test/file.txt");
        assert!(!debouncer.last_events.is_empty());

        debouncer.clear();
        assert!(debouncer.last_events.is_empty());

        // Should process after clear
        assert!(debouncer.should_process("/test/file.txt"));
    }
}
