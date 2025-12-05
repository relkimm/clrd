//! Init command - Initialize clrd in a project

use crate::cli::InitArgs;
use crate::mapper::Mapper;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub async fn run(root: PathBuf, args: InitArgs) -> Result<i32> {
    println!("{}", "Initializing clrd...".bold());
    println!();

    let mapper = Mapper::new(&root);
    let report = mapper.init(args.force)?;

    if !report.created.is_empty() {
        println!("{}", "Created:".green().bold());
        for file in &report.created {
            println!("  {} {}", "+".green(), file);
        }
    }

    if !report.updated.is_empty() {
        println!();
        println!("{}", "Updated:".cyan().bold());
        for file in &report.updated {
            println!("  {} {}", "~".cyan(), file);
        }
    }

    println!();
    println!("{}", "Done!".green().bold());
    println!();
    println!("AI agents can now use {} to clean up dead code.", "clrd".cyan());
    println!("Run {} to detect dead code.", "clrd scan --format json".cyan());

    Ok(0)
}
