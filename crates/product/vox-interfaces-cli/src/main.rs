//! Entry point for the `vox` binary.

mod cli;
mod print_mode;
mod repl;
mod runtime;

use clap::Parser;
use cli::{Cli, Command};
use vox_shared::logging;
use vox_shared::VoxResult;

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("vox: {error}");
            tracing::error!(detail = error.detail(), "command failed");
            1
        }
    };
    std::process::exit(code);
}

/// Dispatch one CLI invocation.
fn run() -> VoxResult<()> {
    let filter = std::env::var("VOX_LOG").unwrap_or_else(|_| "info".to_string());
    logging::init(&filter);
    let cli = Cli::parse();
    let runtime = tokio::runtime::Runtime::new()?;
    match (&cli.command, &cli.prompt) {
        (Some(Command::Config), _) => run_config(&cli),
        (None, Some(prompt)) => runtime.block_on(print_mode::run(&cli, prompt)),
        (None, None) => runtime.block_on(repl::run(&cli)),
    }
}

/// Validate configuration and print the resolved settings.
///
/// Secrets are never printed; only resolved, non-credential settings.
fn run_config(cli: &Cli) -> VoxResult<()> {
    let config = print_mode::resolve_config(cli)?;
    let general = &config.general;
    println!("provider   = {}", general.provider);
    println!("model      = {}", general.model);
    println!(
        "base_url   = {}",
        general.base_url.as_deref().unwrap_or("(provider default)")
    );
    println!("api_key_env= {}", general.api_key_env);
    println!("log        = {}", general.log);
    Ok(())
}
