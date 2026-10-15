//! Command-line interface definition for the `vox` binary.

use clap::{Parser, Subcommand};

/// Top-level command-line parser.
#[derive(Debug, Parser)]
#[command(name = "vox", version, about = "A complete terminal coding agent")]
pub struct Cli {
    /// Run one prompt and exit (print mode).
    #[arg(short = 'p', long = "print", value_name = "PROMPT")]
    pub prompt: Option<String>,
    /// Provider identifier from the provider registry.
    #[arg(long, value_name = "ID", global = true)]
    pub provider: Option<String>,
    /// Model identifier passed to the provider.
    #[arg(long, value_name = "ID", global = true)]
    pub model: Option<String>,
    /// Base URL override for OpenAI-compatible providers.
    #[arg(long, value_name = "URL", global = true)]
    pub base_url: Option<String>,
    /// Environment variable holding the provider API key.
    #[arg(long, value_name = "NAME", global = true)]
    pub api_key_env: Option<String>,
    /// Subcommand to run instead of the default interactive entry point.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate configuration and print the resolved settings.
    Config,
}

impl Cli {
    /// Convert parsed flags into configuration flag overrides.
    pub fn flag_overrides(&self) -> vox_config::FlagOverrides {
        vox_config::FlagOverrides {
            provider: self.provider.clone(),
            model: self.model.clone(),
            base_url: self.base_url.clone(),
            api_key_env: self.api_key_env.clone(),
            log: None,
        }
    }
}
