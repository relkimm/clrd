//! Mapper Module - AI Context File Generation
//!
//! Generates context files for AI agents:
//! - claude.md: For Claude Code (XML/Markdown context)
//! - agent.md: Universal agent guide
//! - .cursorrules: For Cursor editor

pub mod templates;

use crate::types::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub use templates::*;

/// Mapper generates AI context files with dead code reports
pub struct Mapper {
    root: PathBuf,
}

use std::path::PathBuf;

impl Mapper {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Initialize all context files
    pub fn init(&self) -> Result<InitReport> {
        let mut report = InitReport::default();

        // Create claude.md
        if self.create_claude_md()? {
            report.created.push("claude.md".to_string());
        } else {
            report
                .skipped
                .push("claude.md (already exists)".to_string());
        }

        // Create agent.md
        if self.create_agent_md()? {
            report.created.push("agent.md".to_string());
        } else {
            report.skipped.push("agent.md (already exists)".to_string());
        }

        // Create .cursorrules
        if self.create_cursorrules()? {
            report.created.push(".cursorrules".to_string());
        } else {
            report
                .skipped
                .push(".cursorrules (already exists)".to_string());
        }

        Ok(report)
    }

    /// Update context files with scan results
    pub fn update(&self, scan_output: &ScanOutput) -> Result<UpdateReport> {
        let mut report = UpdateReport::default();

        // Update claude.md
        if self.update_claude_md(scan_output)? {
            report.updated.push("claude.md".to_string());
        }

        // Update agent.md
        if self.update_agent_md(scan_output)? {
            report.updated.push("agent.md".to_string());
        }

        // Update .cursorrules
        if self.update_cursorrules(scan_output)? {
            report.updated.push(".cursorrules".to_string());
        }

        Ok(report)
    }

    /// Create claude.md if it doesn't exist
    fn create_claude_md(&self) -> Result<bool> {
        let path = self.root.join("claude.md");
        if path.exists() {
            return Ok(false);
        }

        let content = templates::CLAUDE_MD_TEMPLATE;
        fs::write(&path, content).context("Failed to create claude.md")?;
        Ok(true)
    }

    /// Create agent.md if it doesn't exist
    fn create_agent_md(&self) -> Result<bool> {
        let path = self.root.join("agent.md");
        if path.exists() {
            return Ok(false);
        }

        let content = templates::AGENT_MD_TEMPLATE;
        fs::write(&path, content).context("Failed to create agent.md")?;
        Ok(true)
    }

    /// Create .cursorrules if it doesn't exist
    fn create_cursorrules(&self) -> Result<bool> {
        let path = self.root.join(".cursorrules");
        if path.exists() {
            return Ok(false);
        }

        let content = templates::CURSORRULES_TEMPLATE;
        fs::write(&path, content).context("Failed to create .cursorrules")?;
        Ok(true)
    }

    /// Update claude.md with dead code report
    fn update_claude_md(&self, scan_output: &ScanOutput) -> Result<bool> {
        let path = self.root.join("claude.md");
        let mut content = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            templates::CLAUDE_MD_TEMPLATE.to_string()
        };

        // Remove existing dead code section
        content = self.remove_section(&content, "<!-- CLR:START -->", "<!-- CLR:END -->");

        // Generate new dead code section
        let dead_code_section = self.generate_claude_dead_code_section(scan_output);

        // Append new section
        content.push_str("\n\n");
        content.push_str(&dead_code_section);

        fs::write(&path, content)?;
        Ok(true)
    }

    /// Update agent.md with dead code report
    fn update_agent_md(&self, scan_output: &ScanOutput) -> Result<bool> {
        let path = self.root.join("agent.md");
        let mut content = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            templates::AGENT_MD_TEMPLATE.to_string()
        };

        // Remove existing dead code section
        content = self.remove_section(&content, "<!-- CLR:START -->", "<!-- CLR:END -->");

        // Generate new dead code section
        let dead_code_section = self.generate_agent_dead_code_section(scan_output);

        // Append new section
        content.push_str("\n\n");
        content.push_str(&dead_code_section);

        fs::write(&path, content)?;
        Ok(true)
    }

    /// Update .cursorrules with dead code report
    fn update_cursorrules(&self, scan_output: &ScanOutput) -> Result<bool> {
        let path = self.root.join(".cursorrules");
        let mut content = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            templates::CURSORRULES_TEMPLATE.to_string()
        };

        // Remove existing dead code section
        content = self.remove_section(&content, "# CLR:START", "# CLR:END");

        // Generate new dead code section
        let dead_code_section = self.generate_cursorrules_dead_code_section(scan_output);

        // Append new section
        content.push_str("\n\n");
        content.push_str(&dead_code_section);

        fs::write(&path, content)?;
        Ok(true)
    }

    /// Remove a section between markers
    fn remove_section(&self, content: &str, start_marker: &str, end_marker: &str) -> String {
        if let Some(start) = content.find(start_marker) {
            if let Some(end) = content.find(end_marker) {
                let end = end + end_marker.len();
                let mut result = content[..start].to_string();
                result.push_str(&content[end..]);
                return result.trim_end().to_string();
            }
        }
        content.to_string()
    }

    /// Generate claude.md dead code section (XML format)
    fn generate_claude_dead_code_section(&self, scan_output: &ScanOutput) -> String {
        let mut section = String::new();
        section.push_str("<!-- CLR:START -->\n");
        section.push_str("## Dead Code Report (Generated by clrd)\n\n");
        section.push_str("<dead_code_analysis>\n");
        section.push_str(&format!(
            "  <scan_summary files=\"{}\" issues=\"{}\" high_confidence=\"{}\" />\n",
            scan_output.total_files_scanned,
            scan_output.summary.total_issues,
            scan_output.summary.high_confidence_issues
        ));

        if !scan_output.dead_code.is_empty() {
            section.push_str("  <items>\n");

            for item in &scan_output.dead_code {
                section.push_str(&format!(
                    "    <item kind=\"{}\" confidence=\"{:.2}\">\n",
                    item.kind, item.confidence
                ));
                section.push_str(&format!("      <file>{}</file>\n", item.relative_path));
                section.push_str(&format!(
                    "      <location line=\"{}\" />\n",
                    item.span.start
                ));
                section.push_str(&format!("      <name>{}</name>\n", item.name));
                section.push_str(&format!("      <reason>{}</reason>\n", item.reason));
                section.push_str("      <code><![CDATA[\n");
                section.push_str(&item.code_snippet);
                section.push_str("\n      ]]></code>\n");
                section.push_str("    </item>\n");
            }

            section.push_str("  </items>\n");
        }

        section.push_str("</dead_code_analysis>\n");
        section.push_str("<!-- CLR:END -->");
        section
    }

    /// Generate agent.md dead code section (Markdown format)
    fn generate_agent_dead_code_section(&self, scan_output: &ScanOutput) -> String {
        let mut section = String::new();
        section.push_str("<!-- CLR:START -->\n");
        section.push_str("## 🧹 Dead Code Report\n\n");
        section.push_str(&format!(
            "**Scanned:** {} files | **Issues Found:** {} | **High Confidence:** {}\n\n",
            scan_output.total_files_scanned,
            scan_output.summary.total_issues,
            scan_output.summary.high_confidence_issues
        ));

        if scan_output.dead_code.is_empty() {
            section.push_str("✅ No dead code detected!\n");
        } else {
            section.push_str("### Issues by Category\n\n");

            // Group by kind
            let mut by_kind: std::collections::HashMap<&str, Vec<&DeadCodeItem>> =
                std::collections::HashMap::new();
            for item in &scan_output.dead_code {
                by_kind
                    .entry(match item.kind {
                        DeadCodeKind::UnusedExport => "Unused Exports",
                        DeadCodeKind::UnusedImport => "Unused Imports",
                        DeadCodeKind::ZombieFile => "Zombie Files",
                        DeadCodeKind::UnreachableFunction => "Unreachable Functions",
                        _ => "Other",
                    })
                    .or_default()
                    .push(item);
            }

            for (kind, items) in by_kind {
                section.push_str(&format!("#### {} ({})\n\n", kind, items.len()));
                for item in items {
                    let confidence_emoji = if item.confidence >= 0.8 {
                        "🔴"
                    } else if item.confidence >= 0.5 {
                        "🟡"
                    } else {
                        "🟢"
                    };
                    section.push_str(&format!(
                        "- {} `{}` in `{}` (line {})\n",
                        confidence_emoji, item.name, item.relative_path, item.span.start
                    ));
                }
                section.push('\n');
            }
        }

        section.push_str("<!-- CLR:END -->");
        section
    }

    /// Generate .cursorrules dead code section
    fn generate_cursorrules_dead_code_section(&self, scan_output: &ScanOutput) -> String {
        let mut section = String::new();
        section.push_str("# CLR:START\n");
        section.push_str("# Dead Code Report (Auto-generated by clrd)\n");
        section.push_str("#\n");
        section.push_str(&format!(
            "# Files scanned: {} | Issues: {} | High confidence: {}\n",
            scan_output.total_files_scanned,
            scan_output.summary.total_issues,
            scan_output.summary.high_confidence_issues
        ));
        section.push_str("#\n");

        if !scan_output.dead_code.is_empty() {
            section.push_str("# The following code may be dead and should be reviewed:\n");
            for item in scan_output.dead_code.iter().take(20) {
                section.push_str(&format!(
                    "# - {} in {} (confidence: {:.0}%)\n",
                    item.name,
                    item.relative_path,
                    item.confidence * 100.0
                ));
            }

            if scan_output.dead_code.len() > 20 {
                section.push_str(&format!(
                    "# ... and {} more items\n",
                    scan_output.dead_code.len() - 20
                ));
            }
        }

        section.push_str("# CLR:END");
        section
    }
}

/// Report from init operation
#[derive(Debug, Default)]
pub struct InitReport {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

/// Report from update operation
#[derive(Debug, Default)]
pub struct UpdateReport {
    pub updated: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mapper_init() {
        let dir = tempdir().unwrap();
        let mapper = Mapper::new(dir.path());

        let report = mapper.init().unwrap();
        assert_eq!(report.created.len(), 3);
        assert!(dir.path().join("claude.md").exists());
        assert!(dir.path().join("agent.md").exists());
        assert!(dir.path().join(".cursorrules").exists());
    }

    #[test]
    fn test_remove_section() {
        let mapper = Mapper::new("/tmp");
        let content = "before\n<!-- CLR:START -->\nold content\n<!-- CLR:END -->\nafter";
        let result = mapper.remove_section(content, "<!-- CLR:START -->", "<!-- CLR:END -->");
        assert_eq!(result, "before\nafter");
    }
}
