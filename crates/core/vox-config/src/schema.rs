//! Configuration schema shared by the loader and the CLI.

use serde::Deserialize;

/// Default provider identifier.
pub const DEFAULT_PROVIDER: &str = "openai";
/// Default model identifier.
pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
/// Default environment variable holding the provider API key.
pub const DEFAULT_API_KEY_ENV: &str = "OPENAI_API_KEY";
/// Default logging filter.
pub const DEFAULT_LOG: &str = "info";

/// General agent settings.
///
/// Unknown keys are rejected so configuration typos fail loudly instead of
/// silently changing behaviour.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct GeneralSettings {
    /// Provider identifier from the provider registry.
    pub provider: String,
    /// Model identifier passed to the provider.
    pub model: String,
    /// Optional base URL override for OpenAI-compatible providers.
    pub base_url: Option<String>,
    /// Name of the environment variable holding the provider API key.
    pub api_key_env: String,
    /// Logging filter: error, warn, info, debug, or trace.
    pub log: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            provider: DEFAULT_PROVIDER.to_string(),
            model: DEFAULT_MODEL.to_string(),
            base_url: None,
            api_key_env: DEFAULT_API_KEY_ENV.to_string(),
            log: DEFAULT_LOG.to_string(),
        }
    }
}

/// Top-level configuration document, as written in `vox.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// General agent settings.
    pub general: GeneralSettings,
}
