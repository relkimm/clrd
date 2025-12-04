//! Reference Graph - Cross-file reference tracking
//!
//! Builds a graph of all exports and imports across the codebase
//! to identify unused exports and zombie files.

use crate::types::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Graph of all file references in the project
pub struct ReferenceGraph {
    /// All analyzed files
    nodes: HashMap<PathBuf, ReferenceNode>,
    /// Map from export name to files that export it
    export_index: HashMap<String, Vec<PathBuf>>,
    /// Map from import source to files that import it
    import_index: HashMap<String, Vec<PathBuf>>,
}

impl ReferenceGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            export_index: HashMap::new(),
            import_index: HashMap::new(),
        }
    }

    /// Add a file node to the graph
    pub fn add_node(&mut self, node: ReferenceNode) {
        let file_path = node.file_path.clone();

        // Index exports
        for export in &node.exports {
            self.export_index
                .entry(export.name.clone())
                .or_default()
                .push(file_path.clone());
        }

        // Index imports
        for import in &node.imports {
            self.import_index
                .entry(import.source.clone())
                .or_default()
                .push(file_path.clone());
        }

        self.nodes.insert(file_path, node);
    }

    /// Find all dead code in the graph
    pub fn find_dead_code(
        &self,
        root: &Path,
        confidence_threshold: f64,
    ) -> Result<Vec<DeadCodeItem>> {
        let mut dead_code = Vec::new();

        // Find unused exports
        dead_code.extend(self.find_unused_exports(root, confidence_threshold)?);

        // Find zombie files
        dead_code.extend(self.find_zombie_files(root, confidence_threshold)?);

        // Find unused imports
        dead_code.extend(self.find_unused_imports(root, confidence_threshold)?);

        Ok(dead_code)
    }

    /// Find exports that are never imported
    fn find_unused_exports(
        &self,
        root: &Path,
        _confidence_threshold: f64,
    ) -> Result<Vec<DeadCodeItem>> {
        let mut dead_code = Vec::new();

        // Collect all imported names across the project
        let mut imported_names: HashSet<String> = HashSet::new();
        for node in self.nodes.values() {
            for import in &node.imports {
                imported_names.insert(import.name.clone());
                if let Some(alias) = &import.alias {
                    imported_names.insert(alias.clone());
                }
            }
        }

        // Check each export
        for (file_path, node) in &self.nodes {
            for export in &node.exports {
                // Skip re-exports and wildcard exports
                if export.is_reexport || export.name == "*" {
                    continue;
                }

                // Skip default exports with higher scrutiny (they're often entry points)
                if export.is_default {
                    continue;
                }

                // Check if this export is imported anywhere
                let is_imported = self.is_export_used(file_path, &export.name);

                if !is_imported {
                    let code_snippet = self.get_code_snippet(file_path, &export.span)?;
                    let relative_path = pathdiff::diff_paths(file_path, root)
                        .unwrap_or_else(|| file_path.clone())
                        .to_string_lossy()
                        .to_string();

                    // Determine confidence based on context
                    let confidence = self.calculate_export_confidence(file_path, &export.name);

                    dead_code.push(DeadCodeItem {
                        file_path: file_path.clone(),
                        relative_path,
                        span: export.span,
                        code_snippet,
                        kind: DeadCodeKind::UnusedExport,
                        name: export.name.clone(),
                        reason: format!("Export '{}' has 0 references in the codebase", export.name),
                        confidence,
                        context: Some(DeadCodeContext {
                            possibly_dynamic: self.might_be_dynamic_import(&export.name),
                            in_test_file: self.is_test_file(file_path),
                            public_api: self.is_public_api(file_path, root),
                            partial_references: Vec::new(),
                            doc_comment: None,
                        }),
                    });
                }
            }
        }

        Ok(dead_code)
    }

    /// Find files that are never imported
    fn find_zombie_files(
        &self,
        root: &Path,
        _confidence_threshold: f64,
    ) -> Result<Vec<DeadCodeItem>> {
        let mut dead_code = Vec::new();

        // Collect all imported file paths
        let mut imported_files: HashSet<PathBuf> = HashSet::new();
        for node in self.nodes.values() {
            for import in &node.imports {
                // Resolve the import source to a file path
                if let Some(resolved) = self.resolve_import(&node.file_path, &import.source) {
                    imported_files.insert(resolved);
                }
            }
        }

        // Check each file
        for (file_path, node) in &self.nodes {
            // Skip entry points and config files
            if self.is_likely_entry_point(file_path, root) {
                continue;
            }

            // Check if this file is imported
            if !imported_files.contains(file_path) && !node.exports.is_empty() {
                let relative_path = pathdiff::diff_paths(file_path, root)
                    .unwrap_or_else(|| file_path.clone())
                    .to_string_lossy()
                    .to_string();

                let confidence = if self.is_test_file(file_path) {
                    0.3 // Lower confidence for test files
                } else {
                    0.7
                };

                dead_code.push(DeadCodeItem {
                    file_path: file_path.clone(),
                    relative_path: relative_path.clone(),
                    span: CodeSpan {
                        start: 1,
                        end: 1,
                        col_start: 0,
                        col_end: 0,
                    },
                    code_snippet: format!("// Entire file: {}", relative_path),
                    kind: DeadCodeKind::ZombieFile,
                    name: relative_path,
                    reason: "File is never imported by any other file in the project".to_string(),
                    confidence,
                    context: Some(DeadCodeContext {
                        possibly_dynamic: true,
                        in_test_file: self.is_test_file(file_path),
                        public_api: self.is_public_api(file_path, root),
                        partial_references: Vec::new(),
                        doc_comment: None,
                    }),
                });
            }
        }

        Ok(dead_code)
    }

    /// Find imports that are declared but never used
    fn find_unused_imports(
        &self,
        root: &Path,
        _confidence_threshold: f64,
    ) -> Result<Vec<DeadCodeItem>> {
        let mut dead_code = Vec::new();

        for (file_path, node) in &self.nodes {
            for import in &node.imports {
                // Check if the imported name is used in the file
                let name_to_check = import.alias.as_ref().unwrap_or(&import.name);

                if !node.internal_refs.contains(name_to_check) && name_to_check != "*" {
                    let code_snippet = self.get_code_snippet(file_path, &import.span)?;
                    let relative_path = pathdiff::diff_paths(file_path, root)
                        .unwrap_or_else(|| file_path.clone())
                        .to_string_lossy()
                        .to_string();

                    // Type-only imports have lower confidence (might be used for type annotations)
                    let confidence = if import.is_type_only { 0.6 } else { 0.9 };

                    dead_code.push(DeadCodeItem {
                        file_path: file_path.clone(),
                        relative_path,
                        span: import.span,
                        code_snippet,
                        kind: DeadCodeKind::UnusedImport,
                        name: name_to_check.clone(),
                        reason: format!(
                            "Import '{}' from '{}' is never used in this file",
                            name_to_check, import.source
                        ),
                        confidence,
                        context: None,
                    });
                }
            }
        }

        Ok(dead_code)
    }

    /// Check if an export is used anywhere in the codebase
    fn is_export_used(&self, export_file: &Path, export_name: &str) -> bool {
        for (file_path, node) in &self.nodes {
            if file_path == export_file {
                continue;
            }

            for import in &node.imports {
                // Check if this import comes from the export file
                if let Some(resolved) = self.resolve_import(file_path, &import.source) {
                    if resolved == export_file && import.name == export_name {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Resolve an import source to a file path
    fn resolve_import(&self, from_file: &Path, source: &str) -> Option<PathBuf> {
        // Skip node_modules
        if !source.starts_with('.') && !source.starts_with('/') {
            return None;
        }

        let dir = from_file.parent()?;
        let mut resolved = dir.join(source);

        // Try different extensions
        let extensions = ["", ".ts", ".tsx", ".js", ".jsx", "/index.ts", "/index.tsx", "/index.js"];

        for ext in extensions {
            let candidate = if ext.is_empty() {
                resolved.clone()
            } else {
                PathBuf::from(format!("{}{}", resolved.display(), ext))
            };

            if self.nodes.contains_key(&candidate) {
                return Some(candidate);
            }
        }

        None
    }

    /// Get code snippet from file
    fn get_code_snippet(&self, file_path: &Path, span: &CodeSpan) -> Result<String> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = (span.start as usize).saturating_sub(1);
        let end = (span.end as usize).min(lines.len());

        let snippet: Vec<&str> = lines[start..end].to_vec();

        // Limit snippet size
        if snippet.len() > 10 {
            Ok(format!(
                "{}\n... ({} more lines)",
                snippet[..5].join("\n"),
                snippet.len() - 5
            ))
        } else {
            Ok(snippet.join("\n"))
        }
    }

    /// Calculate confidence score for an unused export
    fn calculate_export_confidence(&self, file_path: &Path, export_name: &str) -> f64 {
        let mut confidence = 0.9;

        // Lower confidence for potential dynamic imports
        if self.might_be_dynamic_import(export_name) {
            confidence -= 0.2;
        }

        // Lower confidence for test files
        if self.is_test_file(file_path) {
            confidence -= 0.3;
        }

        // Lower confidence for files that look like entry points
        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if filename == "index" || filename == "main" || filename == "app" {
            confidence -= 0.2;
        }

        confidence.max(0.1)
    }

    /// Check if a name might be dynamically imported
    fn might_be_dynamic_import(&self, name: &str) -> bool {
        // Common patterns for dynamic imports
        let patterns = ["handler", "middleware", "plugin", "route", "controller", "model"];
        let lower = name.to_lowercase();
        patterns.iter().any(|p| lower.contains(p))
    }

    /// Check if a file is a test file
    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.contains(".test.")
            || path_str.contains(".spec.")
            || path_str.contains("__tests__")
    }

    /// Check if a file is part of the public API
    fn is_public_api(&self, path: &Path, root: &Path) -> bool {
        let relative = pathdiff::diff_paths(path, root).unwrap_or_else(|| path.to_path_buf());
        let relative_str = relative.to_string_lossy();

        // Common public API patterns
        relative_str == "index.ts"
            || relative_str == "index.js"
            || relative_str.starts_with("src/index")
            || relative_str.starts_with("lib/index")
    }

    /// Check if a file is likely an entry point
    fn is_likely_entry_point(&self, path: &Path, root: &Path) -> bool {
        let relative = pathdiff::diff_paths(path, root).unwrap_or_else(|| path.to_path_buf());
        let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let relative_str = relative.to_string_lossy();

        // Entry point patterns
        filename == "index"
            || filename == "main"
            || filename == "app"
            || relative_str.ends_with("index.ts")
            || relative_str.ends_with("index.js")
            || relative_str.contains("pages/") // Next.js pages
            || relative_str.contains("routes/") // Route files
    }
}

impl Default for ReferenceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_graph_creation() {
        let graph = ReferenceGraph::new();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = ReferenceGraph::new();

        let node = ReferenceNode {
            file_path: PathBuf::from("test.ts"),
            exports: vec![ExportedSymbol {
                name: "foo".to_string(),
                kind: SymbolKind::Function,
                span: CodeSpan {
                    start: 1,
                    end: 1,
                    col_start: 0,
                    col_end: 0,
                },
                is_default: false,
                is_reexport: false,
            }],
            imports: vec![],
            internal_refs: vec![],
        };

        graph.add_node(node);
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.export_index.contains_key("foo"));
    }
}
