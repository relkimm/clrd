//! CLI Module - Command Line Interface
//!
//! Implements the clrd commands: init, scan, fix, map

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use commands::*;

/// clrd - AI-native code maintenance tool
///
/// Transparent, Delicate, and Fast dead code detection
#[derive(Parser, Debug)]
#[command(name = "clrd")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Working directory (defaults to current directory)
    #[arg(short = 'C', long, global = true)]
    pub directory: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize clrd in the current project
    ///
    /// Creates context files for AI agents:
    /// - claude.md: For Claude Code
    /// - agent.md: Universal agent guide
    /// - .cursorrules: For Cursor editor
    Init(InitArgs),

    /// Scan for dead code
    ///
    /// Analyzes the codebase and identifies unused exports,
    /// unreachable functions, zombie files, and more.
    Scan(ScanArgs),

    /// Update AI context files with scan results
    ///
    /// Runs a scan and updates claude.md, agent.md, and .cursorrules
    /// with the dead code report.
    Map(MapArgs),

    /// Fix dead code issues
    ///
    /// Remove or comment out dead code based on scan results.
    /// Requires confirmation or --force flag.
    Fix(FixArgs),

    /// Output JSON schema for LLM integration
    Schema,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Force overwrite existing files
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Parser, Debug)]
pub struct ScanArgs {
    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// File extensions to scan (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub extensions: Option<Vec<String>>,

    /// Patterns to ignore (comma-separated glob patterns)
    #[arg(short, long, value_delimiter = ',')]
    pub ignore: Option<Vec<String>>,

    /// Include test files in analysis
    #[arg(long)]
    pub include_tests: bool,

    /// Minimum confidence threshold (0.0 - 1.0)
    #[arg(long, default_value = "0.5")]
    pub confidence: f64,

    /// Output file (for json format)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    /// Human-readable colored output
    #[default]
    Pretty,
    /// JSON output for LLM consumption
    Json,
    /// Compact single-line output
    Compact,
    /// Interactive TUI
    Tui,
}

#[derive(Parser, Debug)]
pub struct MapArgs {
    /// Also run a fresh scan before mapping
    #[arg(long)]
    pub scan: bool,

    /// Minimum confidence threshold for reporting
    #[arg(long, default_value = "0.5")]
    pub confidence: f64,
}

#[derive(Parser, Debug)]
pub struct FixArgs {
    /// Dry run - show what would be removed without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Soft delete - comment out code instead of removing
    #[arg(long)]
    pub soft: bool,

    /// Force removal without confirmation (requires clean git status)
    #[arg(long)]
    pub force: bool,

    /// Only fix items above this confidence threshold
    #[arg(long, default_value = "0.8")]
    pub confidence: f64,

    /// Specific files to fix (if not specified, fixes all)
    #[arg(short, long)]
    pub files: Option<Vec<PathBuf>>,
}

/// Run the CLI with given arguments
pub async fn run_cli(args: Vec<String>) -> Result<i32> {
    let cli = if args.is_empty() {
        // Show help if no args
        Cli::parse_from(["clrd", "--help"])
    } else {
        Cli::parse_from(std::iter::once("clrd".to_string()).chain(args))
    };

    let root = cli
        .directory
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match cli.command {
        Commands::Init(args) => commands::init::run(root, args).await,
        Commands::Scan(args) => commands::scan::run(root, args, cli.verbose).await,
        Commands::Map(args) => commands::map::run(root, args).await,
        Commands::Fix(args) => commands::fix::run(root, args).await,
        Commands::Schema => commands::schema::run().await,
    }
}
