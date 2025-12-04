//! File Walker - Fast parallel file system traversal
//!
//! Uses the `ignore` crate for .gitignore-aware walking
//! with additional custom ignore patterns.

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Walks the file system collecting relevant source files
pub struct FileWalker {
    root: PathBuf,
    extensions: Vec<String>,
    ignore_patterns: GlobSet,
    include_tests: bool,
}

impl FileWalker {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            extensions: Vec::new(),
            ignore_patterns: GlobSet::empty(),
            include_tests: false,
        }
    }

    pub fn with_extensions(mut self, extensions: &[String]) -> Self {
        self.extensions = extensions.to_vec();
        self
    }

    pub fn with_ignore_patterns(mut self, patterns: &[String]) -> Self {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        self.ignore_patterns = builder.build().unwrap_or_else(|_| GlobSet::empty());
        self
    }

    pub fn include_tests(mut self, include: bool) -> Self {
        self.include_tests = include;
        self
    }

    /// Collect all matching files
    pub fn collect_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .parents(true)
            .threads(num_cpus::get())
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check extension
            if !self.has_valid_extension(path) {
                continue;
            }

            // Check ignore patterns
            if self.should_ignore(path) {
                continue;
            }

            // Check if test file (if not including tests)
            if !self.include_tests && self.is_test_file(path) {
                continue;
            }

            files.push(path.to_path_buf());
        }

        Ok(files)
    }

    fn has_valid_extension(&self, path: &Path) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.extensions.iter().any(|e| e == ext))
            .unwrap_or(false)
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.ignore_patterns.is_match(path_str.as_ref())
    }

    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        // Common test file patterns
        path_str.contains(".test.")
            || path_str.contains(".spec.")
            || path_str.contains("__tests__")
            || path_str.contains("__mocks__")
            || path_str.ends_with("_test.ts")
            || path_str.ends_with("_test.js")
            || path_str.ends_with("_spec.ts")
            || path_str.ends_with("_spec.js")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_walker_extensions() {
        let walker = FileWalker::new("/tmp").with_extensions(&["ts".into(), "js".into()]);

        assert!(walker.has_valid_extension(Path::new("foo.ts")));
        assert!(walker.has_valid_extension(Path::new("bar.js")));
        assert!(!walker.has_valid_extension(Path::new("baz.py")));
    }

    #[test]
    fn test_is_test_file() {
        let walker = FileWalker::new("/tmp");

        assert!(walker.is_test_file(Path::new("foo.test.ts")));
        assert!(walker.is_test_file(Path::new("bar.spec.js")));
        assert!(walker.is_test_file(Path::new("__tests__/baz.ts")));
        assert!(!walker.is_test_file(Path::new("utils.ts")));
    }
}
