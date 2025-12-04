//! Fix command - Remove or comment out dead code

use crate::cli::FixArgs;
use crate::scanner::Scanner;
use crate::types::{DeadCodeItem, DeadCodeKind};
use anyhow::{bail, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

pub async fn run(root: PathBuf, args: FixArgs) -> Result<i32> {
    // Check git status if forcing
    if args.force && !is_git_clean(&root)? {
        bail!("Git working directory is not clean. Commit or stash changes before using --force");
    }

    // Run scan first
    println!("{}", "🔍 Scanning for dead code...".bold());
    let scanner = Scanner::new(&root).with_confidence_threshold(args.confidence);
    let scan_output = scanner.scan().await?;

    if scan_output.dead_code.is_empty() {
        println!("{}", "✅ No dead code to fix!".green().bold());
        return Ok(0);
    }

    // Filter by files if specified
    let items: Vec<&DeadCodeItem> = if let Some(ref files) = args.files {
        scan_output
            .dead_code
            .iter()
            .filter(|item| files.iter().any(|f| item.file_path.ends_with(f)))
            .collect()
    } else {
        scan_output.dead_code.iter().collect()
    };

    if items.is_empty() {
        println!("{}", "No matching items to fix.".yellow());
        return Ok(0);
    }

    println!();
    println!(
        "Found {} items to fix (confidence >= {:.0}%)",
        items.len().to_string().cyan(),
        args.confidence * 100.0
    );
    println!();

    // Show preview
    for (i, item) in items.iter().enumerate().take(10) {
        println!(
            "  {}. {} {} ({})",
            i + 1,
            kind_to_action(&item.kind),
            item.name.bold(),
            item.relative_path.dimmed()
        );
    }

    if items.len() > 10 {
        println!("  ... and {} more", items.len() - 10);
    }

    // Dry run mode
    if args.dry_run {
        println!();
        println!("{}", "Dry run mode - no changes made".yellow().bold());
        println!("Run without {} to apply changes", "--dry-run".cyan());
        return Ok(0);
    }

    // Confirmation
    if !args.force {
        println!();
        print!("Apply these changes? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(0);
        }
    }

    // Group items by file for efficient processing
    let mut by_file: HashMap<PathBuf, Vec<&DeadCodeItem>> = HashMap::new();
    for item in &items {
        by_file
            .entry(item.file_path.clone())
            .or_default()
            .push(item);
    }

    // Apply fixes
    let mut fixed = 0;
    let mut errors = 0;

    for (file_path, file_items) in by_file {
        match apply_fixes(&file_path, &file_items, args.soft) {
            Ok(count) => {
                fixed += count;
                println!("  {} Fixed {} items in {}", "✓".green(), count, file_path.display());
            }
            Err(e) => {
                errors += 1;
                println!(
                    "  {} Error in {}: {}",
                    "✗".red(),
                    file_path.display(),
                    e
                );
            }
        }
    }

    println!();
    println!(
        "{}",
        format!("Fixed {} items with {} errors", fixed, errors).bold()
    );

    if errors > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn is_git_clean(root: &PathBuf) -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output();

    match output {
        Ok(output) => Ok(output.stdout.is_empty()),
        Err(_) => Ok(true), // Not a git repo, allow operation
    }
}

fn apply_fixes(file_path: &PathBuf, items: &[&DeadCodeItem], soft: bool) -> Result<usize> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Sort items by line number in reverse order to avoid offset issues
    let mut sorted_items = items.to_vec();
    sorted_items.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    for item in &sorted_items {
        let start = (item.span.start as usize).saturating_sub(1);
        let end = (item.span.end as usize).min(new_lines.len());

        if soft {
            // Comment out the code
            for i in start..end {
                new_lines[i] = format!("// [clr] {}", new_lines[i]);
            }
        } else {
            // Remove the lines
            // Mark for removal
            for i in start..end {
                new_lines[i] = "\x00REMOVE\x00".to_string();
            }
        }
    }

    // Remove marked lines
    let final_lines: Vec<String> = new_lines
        .into_iter()
        .filter(|l| l != "\x00REMOVE\x00")
        .collect();

    fs::write(file_path, final_lines.join("\n"))?;

    Ok(sorted_items.len())
}

fn kind_to_action(kind: &DeadCodeKind) -> &'static str {
    use DeadCodeKind::*;
    match kind {
        UnusedExport => "Remove export",
        UnreachableFunction => "Remove function",
        UnusedVariable => "Remove variable",
        UnusedImport => "Remove import",
        ZombieFile => "Delete file",
        UnusedType => "Remove type",
        UnusedClass => "Remove class",
        UnusedEnum => "Remove enum",
        DeadBranch => "Remove branch",
    }
}
