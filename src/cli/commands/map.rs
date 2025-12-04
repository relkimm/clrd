//! Map command - Update AI context files

use crate::cli::MapArgs;
use crate::mapper::Mapper;
use crate::scanner::Scanner;
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

pub async fn run(root: PathBuf, args: MapArgs) -> Result<i32> {
    println!("{}", "🗺️  Updating AI context files...".bold());
    println!();

    // First, run a scan to get current state
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")?,
    );
    spinner.set_message("Scanning codebase...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let scanner = Scanner::new(&root).with_confidence_threshold(args.confidence);
    let scan_output = scanner.scan().await?;

    spinner.finish_and_clear();
    println!(
        "  {} Scan complete: {} files, {} issues",
        "✓".green(),
        scan_output.total_files_scanned,
        scan_output.summary.total_issues
    );

    // Update context files
    let mapper = Mapper::new(&root);
    let report = mapper.update(&scan_output)?;

    println!();
    println!("{}", "Updated files:".green().bold());
    for file in &report.updated {
        println!("  {} {}", "✓".green(), file);
    }

    println!();
    println!("{}", "Context files now contain:".bold());
    println!(
        "  • Dead code report with {} issues",
        scan_output.summary.total_issues
    );
    println!(
        "  • {} high confidence items",
        scan_output.summary.high_confidence_issues
    );
    println!("  • Code snippets for AI review");

    println!();
    println!(
        "Your AI agent can now see the dead code report in {} and {}",
        "claude.md".cyan(),
        "agent.md".cyan()
    );

    Ok(0)
}
