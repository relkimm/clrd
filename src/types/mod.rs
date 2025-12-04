//! Core types for clr
//!
//! This module defines the data structures used throughout the codebase,
//! including JSON schemas for LLM communication.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// The kind of dead code detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeadCodeKind {
    /// Exported symbol with no external references
    UnusedExport,
    /// Function that is never called
    UnreachableFunction,
    /// Variable that is declared but never used
    UnusedVariable,
    /// Import that is never used
    UnusedImport,
    /// File with no imports from other files
    ZombieFile,
    /// Type/Interface that is never referenced
    UnusedType,
    /// Class that is never instantiated or extended
    UnusedClass,
    /// Enum that is never used
    UnusedEnum,
    /// Dead branch in conditional logic
    DeadBranch,
}

impl std::fmt::Display for DeadCodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeadCodeKind::UnusedExport => write!(f, "unused_export"),
            DeadCodeKind::UnreachableFunction => write!(f, "unreachable_function"),
            DeadCodeKind::UnusedVariable => write!(f, "unused_variable"),
            DeadCodeKind::UnusedImport => write!(f, "unused_import"),
            DeadCodeKind::ZombieFile => write!(f, "zombie_file"),
            DeadCodeKind::UnusedType => write!(f, "unused_type"),
            DeadCodeKind::UnusedClass => write!(f, "unused_class"),
            DeadCodeKind::UnusedEnum => write!(f, "unused_enum"),
            DeadCodeKind::DeadBranch => write!(f, "dead_branch"),
        }
    }
}

/// Span information for code location
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct CodeSpan {
    /// Starting line (1-indexed)
    pub start: u32,
    /// Ending line (1-indexed)
    pub end: u32,
    /// Starting column (0-indexed)
    pub col_start: u32,
    /// Ending column (0-indexed)
    pub col_end: u32,
}

/// A detected piece of dead code
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeadCodeItem {
    /// Absolute path to the file
    pub file_path: PathBuf,
    /// Relative path from project root
    pub relative_path: String,
    /// Location in the file
    pub span: CodeSpan,
    /// The actual code snippet (for LLM context)
    pub code_snippet: String,
    /// Type of dead code
    pub kind: DeadCodeKind,
    /// Name of the symbol (function name, variable name, etc.)
    pub name: String,
    /// Human-readable reason for flagging
    pub reason: String,
    /// Confidence score (0.0 - 1.0)
    /// Lower confidence items may need LLM judgment
    pub confidence: f64,
    /// Additional context for LLM decision making
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<DeadCodeContext>,
}

/// Additional context to help LLM make decisions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeadCodeContext {
    /// Is this potentially a dynamic import/require?
    pub possibly_dynamic: bool,
    /// Is this in a test file?
    pub in_test_file: bool,
    /// Is this exported from package entry point?
    pub public_api: bool,
    /// Files that import this (if any partial references exist)
    pub partial_references: Vec<String>,
    /// JSDoc or comment hints suggesting intentional code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_comment: Option<String>,
}

/// Result of a scan operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScanOutput {
    /// Version of clr that generated this output
    pub version: String,
    /// Root directory that was scanned
    pub root: String,
    /// Timestamp of scan
    pub timestamp: String,
    /// All detected dead code items
    pub dead_code: Vec<DeadCodeItem>,
    /// Total files scanned
    pub total_files_scanned: u32,
    /// Total lines of code analyzed
    pub total_lines: u64,
    /// Scan duration
    #[serde(with = "duration_serde")]
    pub scan_duration: Duration,
    /// Summary statistics
    pub summary: ScanSummary,
}

/// Summary statistics from a scan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScanSummary {
    pub unused_exports: u32,
    pub unreachable_functions: u32,
    pub unused_variables: u32,
    pub unused_imports: u32,
    pub zombie_files: u32,
    pub unused_types: u32,
    pub total_issues: u32,
    pub high_confidence_issues: u32,
    pub low_confidence_issues: u32,
}

impl ScanSummary {
    pub fn new() -> Self {
        Self {
            unused_exports: 0,
            unreachable_functions: 0,
            unused_variables: 0,
            unused_imports: 0,
            zombie_files: 0,
            unused_types: 0,
            total_issues: 0,
            high_confidence_issues: 0,
            low_confidence_issues: 0,
        }
    }

    pub fn add(&mut self, item: &DeadCodeItem) {
        self.total_issues += 1;

        if item.confidence >= 0.8 {
            self.high_confidence_issues += 1;
        } else {
            self.low_confidence_issues += 1;
        }

        match item.kind {
            DeadCodeKind::UnusedExport => self.unused_exports += 1,
            DeadCodeKind::UnreachableFunction => self.unreachable_functions += 1,
            DeadCodeKind::UnusedVariable => self.unused_variables += 1,
            DeadCodeKind::UnusedImport => self.unused_imports += 1,
            DeadCodeKind::ZombieFile => self.zombie_files += 1,
            DeadCodeKind::UnusedType | DeadCodeKind::UnusedClass | DeadCodeKind::UnusedEnum => {
                self.unused_types += 1
            }
            DeadCodeKind::DeadBranch => {}
        }
    }
}

impl Default for ScanSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for clr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClrConfig {
    /// File extensions to scan
    pub extensions: Vec<String>,
    /// Patterns to ignore (glob patterns)
    pub ignore_patterns: Vec<String>,
    /// Whether to include test files in analysis
    pub include_tests: bool,
    /// Minimum confidence threshold for reporting
    pub confidence_threshold: f64,
    /// Output format preferences
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Generate agent.md
    pub agent_md: bool,
    /// Generate claude.md
    pub claude_md: bool,
    /// Generate .cursorrules
    pub cursorrules: bool,
}

impl Default for ClrConfig {
    fn default() -> Self {
        Self {
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
                "**/coverage/**".into(),
                "**/*.min.js".into(),
                "**/*.bundle.js".into(),
            ],
            include_tests: false,
            confidence_threshold: 0.5,
            output: OutputConfig {
                agent_md: true,
                claude_md: true,
                cursorrules: true,
            },
        }
    }
}

/// Reference graph node
#[derive(Debug, Clone)]
pub struct ReferenceNode {
    pub file_path: PathBuf,
    pub exports: Vec<ExportedSymbol>,
    pub imports: Vec<ImportedSymbol>,
    pub internal_refs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExportedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: CodeSpan,
    pub is_default: bool,
    pub is_reexport: bool,
}

#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    pub name: String,
    pub alias: Option<String>,
    pub source: String,
    pub is_type_only: bool,
    pub span: CodeSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Variable,
    Type,
    Interface,
    Enum,
    Const,
    Let,
    Namespace,
}

/// Custom serialization for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// LLM judgment request format
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlmJudgmentRequest {
    pub items: Vec<DeadCodeItem>,
    pub project_context: ProjectContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectContext {
    pub name: String,
    pub framework: Option<String>,
    pub package_json_main: Option<String>,
    pub package_json_exports: Vec<String>,
}

/// LLM judgment response format
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlmJudgmentResponse {
    pub confirmed: Vec<ConfirmedDeadCode>,
    pub rejected: Vec<RejectedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfirmedDeadCode {
    pub file_path: String,
    pub name: String,
    pub action: RemovalAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemovalAction {
    Delete,
    CommentOut,
    MoveToTrash,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RejectedItem {
    pub file_path: String,
    pub name: String,
    pub reason: String,
}
