//! Glob pattern matching for file paths

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

use crate::error::{WatcherError, WatcherResult};

/// Pattern matcher for filtering file paths against glob patterns
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    glob_set: GlobSet,
    patterns: Vec<String>,
}

impl PatternMatcher {
    /// Create a new pattern matcher from a list of glob patterns
    ///
    /// If the patterns list is empty, the matcher will match all files.
    ///
    /// # Examples
    ///
    /// ```
    /// use leaf_watcher::PatternMatcher;
    ///
    /// let matcher = PatternMatcher::new(&["*.pdf".to_string(), "*.txt".to_string()]).unwrap();
    /// assert!(matcher.matches("document.pdf"));
    /// assert!(matcher.matches("notes.txt"));
    /// assert!(!matcher.matches("image.png"));
    /// ```
    pub fn new(patterns: &[String]) -> WatcherResult<Self> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let glob = Glob::new(pattern).map_err(|e| WatcherError::InvalidPattern {
                pattern: pattern.clone(),
                reason: e.to_string(),
            })?;
            builder.add(glob);
        }

        let glob_set = builder.build().map_err(|e| WatcherError::InvalidPattern {
            pattern: patterns.join(", "),
            reason: e.to_string(),
        })?;

        Ok(Self {
            glob_set,
            patterns: patterns.to_vec(),
        })
    }

    /// Create a matcher that matches all files
    pub fn match_all() -> Self {
        Self {
            glob_set: GlobSet::empty(),
            patterns: Vec::new(),
        }
    }

    /// Check if a path matches any of the patterns
    ///
    /// If no patterns are configured, matches all files.
    pub fn matches(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        // If no patterns, match everything
        if self.patterns.is_empty() {
            return true;
        }

        // Get just the filename for matching
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        // Try matching against the full path and just the filename
        self.glob_set.is_match(path) || self.glob_set.is_match(filename)
    }

    /// Get the configured patterns
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Check if this matcher has any patterns configured
    pub fn has_patterns(&self) -> bool {
        !self.patterns.is_empty()
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::match_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matcher_basic() {
        let matcher = PatternMatcher::new(&["*.pdf".to_string(), "*.txt".to_string()]).unwrap();

        assert!(matcher.matches("document.pdf"));
        assert!(matcher.matches("notes.txt"));
        assert!(!matcher.matches("image.png"));
    }

    #[test]
    fn test_pattern_matcher_full_path() {
        let matcher = PatternMatcher::new(&["*.pdf".to_string()]).unwrap();

        assert!(matcher.matches("/home/user/documents/report.pdf"));
        assert!(matcher.matches("./data/file.pdf"));
        assert!(!matcher.matches("/home/user/image.png"));
    }

    #[test]
    fn test_pattern_matcher_recursive() {
        let matcher = PatternMatcher::new(&["**/*.pdf".to_string()]).unwrap();

        assert!(matcher.matches("/deep/nested/path/doc.pdf"));
        assert!(matcher.matches("simple.pdf"));
    }

    #[test]
    fn test_pattern_matcher_empty_matches_all() {
        let matcher = PatternMatcher::new(&[]).unwrap();

        assert!(matcher.matches("any.file"));
        assert!(matcher.matches("/any/path/file.txt"));
    }

    #[test]
    fn test_pattern_matcher_match_all() {
        let matcher = PatternMatcher::match_all();

        assert!(matcher.matches("anything"));
        assert!(matcher.matches("/path/to/anything.xyz"));
    }

    #[test]
    fn test_pattern_matcher_invalid_pattern() {
        let result = PatternMatcher::new(&["[invalid".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_matcher_complex_patterns() {
        let matcher = PatternMatcher::new(&[
            "*.{jpg,jpeg,png,gif}".to_string(),
            "doc_*.pdf".to_string(),
        ])
        .unwrap();

        assert!(matcher.matches("photo.jpg"));
        assert!(matcher.matches("image.png"));
        assert!(matcher.matches("doc_2024.pdf"));
        assert!(!matcher.matches("random.pdf"));
    }
}
