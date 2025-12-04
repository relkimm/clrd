//! Scan command - Detect dead code

use crate::cli::{OutputFormat, ScanArgs};
use crate::scanner::Scanner;
use crate::tui;
use crate::types::ScanOutput;
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub async fn run(root: PathBuf, args: ScanArgs, verbose: bool) -> Result<i32> {
    // Show progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")?,
    );
    spinner.set_message("Scanning for dead code...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    // Build scanner
    let mut scanner = Scanner::new(&root).with_confidence_threshold(args.confidence);

    if let Some(extensions) = args.extensions {
        scanner = scanner.with_extensions(extensions);
    }

    if let Some(ignore) = args.ignore {
        scanner = scanner.with_ignore_patterns(ignore);
    }

    scanner = scanner.include_tests(args.include_tests);

    // Run scan
    let result = scanner.scan().await?;
    spinner.finish_and_clear();

    // Output based on format
    match args.format {
        OutputFormat::Pretty => print_pretty(&result, verbose),
        OutputFormat::Json => print_json(&result, args.output)?,
        OutputFormat::Compact => print_compact(&result),
        OutputFormat::Tui => tui::run_tui(&result)?,
    }

    // Return exit code based on findings
    if result.summary.high_confidence_issues > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn print_pretty(result: &ScanOutput, verbose: bool) {
    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!("{}", " 🧹 clrd - Dead Code Report".bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    // Summary
    println!("{}", "Summary".bold().underline());
    println!(
        "  Files scanned:     {}",
        result.total_files_scanned.to_string().cyan()
    );
    println!(
        "  Scan duration:     {}ms",
        result.scan_duration_ms.to_string().cyan()
    );
    println!(
        "  Total issues:      {}",
        colorize_count(result.summary.total_issues)
    );
    println!(
        "  High confidence:   {}",
        colorize_count(result.summary.high_confidence_issues)
    );
    println!();

    if result.dead_code.is_empty() {
        println!("{}", "✅ No dead code detected!".green().bold());
        return;
    }

    // Group by kind
    println!("{}", "Issues by Category".bold().underline());
    println!();

    if result.summary.unused_exports > 0 {
        println!(
            "  {} Unused Exports: {}",
            "●".red(),
            result.summary.unused_exports
        );
    }
    if result.summary.unused_imports > 0 {
        println!(
            "  {} Unused Imports: {}",
            "●".yellow(),
            result.summary.unused_imports
        );
    }
    if result.summary.zombie_files > 0 {
        println!(
            "  {} Zombie Files: {}",
            "●".magenta(),
            result.summary.zombie_files
        );
    }
    if result.summary.unreachable_functions > 0 {
        println!(
            "  {} Unreachable Functions: {}",
            "●".blue(),
            result.summary.unreachable_functions
        );
    }

    println!();
    println!("{}", "Details".bold().underline());
    println!();

    // Show details
    let items_to_show = if verbose {
        result.dead_code.len()
    } else {
        result.dead_code.len().min(20)
    };

    for (i, item) in result.dead_code.iter().take(items_to_show).enumerate() {
        let confidence_bar = confidence_to_bar(item.confidence);
        let kind_icon = kind_to_icon(&item.kind);

        println!(
            "{:>3}. {} {} {}",
            i + 1,
            kind_icon,
            item.name.bold(),
            confidence_bar
        );
        println!("     {} {}", "→".dimmed(), item.relative_path.dimmed());
        println!(
            "     {} Line {}",
            "↳".dimmed(),
            item.span.start.to_string().cyan()
        );

        if verbose {
            println!("     {}", item.reason.dimmed());
            println!("     {}", "─".repeat(40).dimmed());
            for line in item.code_snippet.lines().take(5) {
                println!("     {}", line.dimmed());
            }
        }
        println!();
    }

    if result.dead_code.len() > items_to_show {
        println!(
            "  {} ... and {} more items (use {} for full list)",
            "⋯".dimmed(),
            result.dead_code.len() - items_to_show,
            "--verbose".cyan()
        );
    }

    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "Run {} to output JSON for LLM analysis",
        "clrd scan --format json".cyan()
    );
    println!("Run {} to update AI context files", "clrd map".cyan());
}

fn print_json(result: &ScanOutput, output: Option<PathBuf>) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;

    if let Some(path) = output {
        fs::write(&path, &json)?;
        eprintln!("Output written to: {}", path.display());
    } else {
        println!("{}", json);
    }

    Ok(())
}

fn print_compact(result: &ScanOutput) {
    println!(
        "clrd: {} files scanned, {} issues ({} high confidence)",
        result.total_files_scanned,
        result.summary.total_issues,
        result.summary.high_confidence_issues
    );

    for item in &result.dead_code {
        println!(
            "  {}:{} {} ({:.0}%)",
            item.relative_path,
            item.span.start,
            item.name,
            item.confidence * 100.0
        );
    }
}

fn colorize_count(count: u32) -> String {
    if count == 0 {
        count.to_string().green().to_string()
    } else if count < 5 {
        count.to_string().yellow().to_string()
    } else {
        count.to_string().red().to_string()
    }
}

fn confidence_to_bar(confidence: f64) -> String {
    let filled = (confidence * 5.0).round() as usize;
    let empty = 5 - filled;

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if confidence >= 0.8 {
        bar.red().to_string()
    } else if confidence >= 0.5 {
        bar.yellow().to_string()
    } else {
        bar.green().to_string()
    }
}

fn kind_to_icon(kind: &crate::types::DeadCodeKind) -> &'static str {
    use crate::types::DeadCodeKind::*;
    match kind {
        UnusedExport => "📤",
        UnreachableFunction => "🔒",
        UnusedVariable => "📦",
        UnusedImport => "📥",
        ZombieFile => "🧟",
        UnusedType => "📝",
        UnusedClass => "🏛️",
        UnusedEnum => "🔢",
        DeadBranch => "🌿",
    }
}
