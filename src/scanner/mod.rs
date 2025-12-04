//! Scanner Module - High-speed dead code detection
//!
//! Uses Oxc parser and Rayon for parallel processing to achieve
//! maximum performance scanning JavaScript/TypeScript codebases.

mod analyzer;
mod file_walker;
mod reference_graph;

pub use analyzer::AstAnalyzer;
pub use file_walker::FileWalker;
pub use reference_graph::ReferenceGraph;

use crate::types::*;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The main scanner that orchestrates dead code detection
pub struct Scanner {
    root: PathBuf,
    extensions: Vec<String>,
    ignore_patterns: Vec<String>,
    include_tests: bool,
    confidence_threshold: f64,
}

impl Scanner {
    /// Create a new scanner for the given root directory
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            extensions: vec![
                "ts".into(),
                "tsx".into(),
                "js".into(),
                "jsx".into(),
                "mjs".into(),
                "cjs".into(),
            ],
            ignore_patterns: vec![
                "**/node_modules/**".into(),
                "**/dist/**".into(),
                "**/build/**".into(),
                "**/.git/**".into(),
            ],
            include_tests: false,
            confidence_threshold: 0.5,
        }
    }

    /// Set file extensions to scan
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        if !extensions.is_empty() {
            self.extensions = extensions;
        }
        self
    }

    /// Set patterns to ignore
    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        if !patterns.is_empty() {
            self.ignore_patterns = patterns;
        }
        self
    }

    /// Whether to include test files
    pub fn include_tests(mut self, include: bool) -> Self {
        self.include_tests = include;
        self
    }

    /// Set minimum confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Execute the scan and return results
    pub async fn scan(&self) -> Result<ScanOutput> {
        let start = Instant::now();

        // Phase 1: Walk the file system and collect files
        tracing::info!("Phase 1: Collecting files from {:?}", self.root);
        let walker = FileWalker::new(&self.root)
            .with_extensions(&self.extensions)
            .with_ignore_patterns(&self.ignore_patterns)
            .include_tests(self.include_tests);

        let files = walker.collect_files()?;
        let total_files = files.len() as u32;
        tracing::info!("Found {} files to analyze", total_files);

        // Phase 2: Parse all files in parallel and build reference graph
        tracing::info!("Phase 2: Building reference graph");
        let graph = Arc::new(Mutex::new(ReferenceGraph::new()));
        let total_lines = Arc::new(Mutex::new(0u64));

        files.par_iter().for_each(|file_path| {
            match AstAnalyzer::analyze_file(file_path) {
                Ok(node) => {
                    let lines = node.exports.len() + node.imports.len();
                    *total_lines.lock().unwrap() += lines as u64;
                    graph.lock().unwrap().add_node(node);
                }
                Err(e) => {
                    tracing::warn!("Failed to analyze {:?}: {}", file_path, e);
                }
            }
        });

        // Phase 3: Detect dead code
        tracing::info!("Phase 3: Detecting dead code");
        let graph = Arc::try_unwrap(graph)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap graph"))?
            .into_inner()?;

        let dead_code = graph.find_dead_code(&self.root, self.confidence_threshold)?;

        // Build summary
        let mut summary = ScanSummary::new();
        for item in &dead_code {
            summary.add(item);
        }

        let scan_duration = start.elapsed();
        let total_lines = Arc::try_unwrap(total_lines)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap lines counter"))?
            .into_inner()?;

        Ok(ScanOutput {
            version: env!("CARGO_PKG_VERSION").to_string(),
            root: self.root.to_string_lossy().to_string(),
            timestamp: chrono_lite_now(),
            dead_code,
            total_files_scanned: total_files,
            total_lines,
            scan_duration,
            summary,
        })
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scanner_creation() {
        let scanner = Scanner::new("/tmp")
            .with_extensions(vec!["ts".into()])
            .include_tests(true);

        assert!(scanner.include_tests);
        assert_eq!(scanner.extensions, vec!["ts".to_string()]);
    }
}
