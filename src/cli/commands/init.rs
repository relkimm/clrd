//! Init command - Initialize clr in a project

use crate::cli::InitArgs;
use crate::mapper::Mapper;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub async fn run(root: PathBuf, _args: InitArgs) -> Result<i32> {
    println!("{}", "🧹 Initializing clr...".bold());
    println!();

    let mapper = Mapper::new(&root);
    let report = mapper.init()?;

    if !report.created.is_empty() {
        println!("{}", "Created:".green().bold());
        for file in &report.created {
            println!("  {} {}", "✓".green(), file);
        }
    }

    if !report.skipped.is_empty() {
        println!();
        println!("{}", "Skipped:".yellow().bold());
        for file in &report.skipped {
            println!("  {} {}", "○".yellow(), file);
        }
    }

    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Edit the context files to describe your project");
    println!("  2. Run {} to detect dead code", "clr scan".cyan());
    println!("  3. Run {} to update context files with results", "clr map".cyan());

    Ok(0)
}
