//! clrd CLI binary entry point
//!
//! This is the standalone Rust binary for direct execution.
//! For npm distribution, the NAPI bindings in lib.rs are used instead.

use clrd::cli;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("clr=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();

    match cli::run_cli(args).await {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
