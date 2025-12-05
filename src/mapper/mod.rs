//! Mapper Module - AI Context File Generation
//!
//! Creates clrd.md with usage instructions and adds references to existing AI context files.

pub mod templates;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub use templates::*;

/// Marker to check if clrd reference already exists
const CLRD_REFERENCE_MARKER: &str = "clrd.md";

/// Mapper generates AI context files
pub struct Mapper {
    root: PathBuf,
}

impl Mapper {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Initialize clrd context
    /// - Always creates/updates clrd.md
    /// - Adds reference to existing claude.md, agent.md, .cursorrules
    pub fn init(&self, force: bool) -> Result<InitReport> {
        let mut report = InitReport::default();

        // Always create clrd.md
        self.create_clrd_md(force)?;
        report.created.push("clrd.md".to_string());

        // Add reference to existing files
        if self.add_reference_to_claude_md()? {
            report.updated.push("claude.md (added clrd.md reference)".to_string());
        }

        if self.add_reference_to_agent_md()? {
            report.updated.push("agent.md (added clrd.md reference)".to_string());
        }

        if self.add_reference_to_cursorrules()? {
            report.updated.push(".cursorrules (added clrd.md reference)".to_string());
        }

        Ok(report)
    }

    /// Create clrd.md with usage instructions
    fn create_clrd_md(&self, force: bool) -> Result<()> {
        let path = self.root.join("clrd.md");

        if path.exists() && !force {
            // Update anyway - clrd.md is our file
        }

        let content = templates::CLRD_MD_TEMPLATE;
        fs::write(&path, content).context("Failed to create clrd.md")?;
        Ok(())
    }

    /// Add reference to claude.md if it exists and doesn't already have one
    fn add_reference_to_claude_md(&self) -> Result<bool> {
        let path = self.root.join("claude.md");
        if !path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&path)?;

        // Check if reference already exists
        if content.contains(CLRD_REFERENCE_MARKER) {
            return Ok(false);
        }

        // Append reference
        let mut new_content = content;
        new_content.push_str(templates::CLAUDE_MD_REFERENCE);
        fs::write(&path, new_content)?;

        Ok(true)
    }

    /// Add reference to agent.md if it exists and doesn't already have one
    fn add_reference_to_agent_md(&self) -> Result<bool> {
        let path = self.root.join("agent.md");
        if !path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&path)?;

        // Check if reference already exists
        if content.contains(CLRD_REFERENCE_MARKER) {
            return Ok(false);
        }

        // Append reference
        let mut new_content = content;
        new_content.push_str(templates::AGENT_MD_REFERENCE);
        fs::write(&path, new_content)?;

        Ok(true)
    }

    /// Add reference to .cursorrules if it exists and doesn't already have one
    fn add_reference_to_cursorrules(&self) -> Result<bool> {
        let path = self.root.join(".cursorrules");
        if !path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&path)?;

        // Check if reference already exists
        if content.contains(CLRD_REFERENCE_MARKER) {
            return Ok(false);
        }

        // Append reference
        let mut new_content = content;
        new_content.push_str(templates::CURSORRULES_REFERENCE);
        fs::write(&path, new_content)?;

        Ok(true)
    }
}

/// Report from init operation
#[derive(Debug, Default)]
pub struct InitReport {
    pub created: Vec<String>,
    pub updated: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mapper_init_fresh() {
        let dir = tempdir().unwrap();
        let mapper = Mapper::new(dir.path());

        let report = mapper.init(false).unwrap();

        // Only clrd.md should be created
        assert_eq!(report.created.len(), 1);
        assert!(report.created.contains(&"clrd.md".to_string()));
        assert!(dir.path().join("clrd.md").exists());

        // No updates since no existing files
        assert!(report.updated.is_empty());
    }

    #[test]
    fn test_mapper_init_with_existing_claude_md() {
        let dir = tempdir().unwrap();

        // Create existing claude.md
        fs::write(dir.path().join("claude.md"), "# My Project\n\nSome content").unwrap();

        let mapper = Mapper::new(dir.path());
        let report = mapper.init(false).unwrap();

        // clrd.md created
        assert!(report.created.contains(&"clrd.md".to_string()));

        // claude.md updated with reference
        assert!(report.updated.iter().any(|s| s.contains("claude.md")));

        // Check reference was added
        let claude_content = fs::read_to_string(dir.path().join("claude.md")).unwrap();
        assert!(claude_content.contains("clrd.md"));
    }

    #[test]
    fn test_mapper_init_no_duplicate_reference() {
        let dir = tempdir().unwrap();

        // Create claude.md with existing reference
        fs::write(dir.path().join("claude.md"), "# My Project\n\nSee clrd.md for info").unwrap();

        let mapper = Mapper::new(dir.path());
        let report = mapper.init(false).unwrap();

        // claude.md should NOT be in updated (reference already exists)
        assert!(!report.updated.iter().any(|s| s.contains("claude.md")));
    }
}
