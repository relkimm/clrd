//! clrd - AI-native code maintenance tool
//!
//! "Transparent, Delicate, and Fast"
//!
//! This crate provides high-speed dead code detection using Rust and Oxc,
//! with intelligent judgment handoff to LLMs for avoiding false positives.

#![allow(dead_code)]

pub mod cli;
pub mod mapper;
pub mod scanner;
pub mod tui;
pub mod types;

// Re-exports for external use
pub use mapper::Mapper;
pub use scanner::Scanner;
pub use types::*;

// NAPI bindings - only compiled when napi feature is enabled
#[cfg(feature = "napi")]
mod napi_bindings {
    use super::*;
    use napi_derive::napi;
    use std::sync::OnceLock;

    static INIT: OnceLock<()> = OnceLock::new();

    fn init_logger() {
        INIT.get_or_init(|| {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive("clrd=info".parse().unwrap()),
                )
                .with_target(false)
                .init();
        });
    }

    /// Entry point for Node.js via NAPI
    /// Runs the CLI with the given arguments
    #[napi]
    pub async fn run(args: Vec<String>) -> napi::Result<i32> {
        init_logger();

        match cli::run_cli(args).await {
            Ok(code) => Ok(code),
            Err(e) => {
                eprintln!("Error: {e}");
                Ok(1)
            }
        }
    }

    /// Scan a directory for dead code (NAPI export for programmatic use)
    #[napi(object)]
    #[derive(Debug, Clone)]
    pub struct ScanOptions {
        pub root: String,
        pub extensions: Vec<String>,
        pub ignore_patterns: Vec<String>,
        pub include_tests: bool,
    }

    #[napi(object)]
    #[derive(Debug, Clone)]
    pub struct ScanResultItem {
        pub file_path: String,
        pub line_start: u32,
        pub line_end: u32,
        pub code_snippet: String,
        pub kind: String,
        pub name: String,
        pub reason: String,
        pub confidence: f64,
    }

    #[napi(object)]
    #[derive(Debug, Clone)]
    pub struct ScanResult {
        pub items: Vec<ScanResultItem>,
        pub total_files_scanned: u32,
        pub scan_duration_ms: i64,
    }

    /// Programmatic scan API for Node.js consumers
    #[napi]
    pub async fn scan(options: ScanOptions) -> napi::Result<ScanResult> {
        init_logger();

        let scanner = Scanner::new(&options.root)
            .with_extensions(options.extensions)
            .with_ignore_patterns(options.ignore_patterns)
            .include_tests(options.include_tests);

        let result = scanner
            .scan()
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        Ok(ScanResult {
            items: result
                .dead_code
                .into_iter()
                .map(|item| ScanResultItem {
                    file_path: item.file_path.to_string_lossy().to_string(),
                    line_start: item.span.start,
                    line_end: item.span.end,
                    code_snippet: item.code_snippet,
                    kind: item.kind.to_string(),
                    name: item.name,
                    reason: item.reason,
                    confidence: item.confidence,
                })
                .collect(),
            total_files_scanned: result.total_files_scanned,
            scan_duration_ms: result.scan_duration_ms as i64,
        })
    }
}

#[cfg(feature = "napi")]
pub use napi_bindings::*;
