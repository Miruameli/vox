//! Layered configuration loading with strict validation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use vox_shared::VoxResult;

use crate::files::merge_files;
use crate::schema::Config;
use crate::validation::validate;

/// Project configuration file name discovered from the working directory.
pub const PROJECT_CONFIG_NAME: &str = "vox.toml";

/// Environment variable prefix honoured by the loader.
const ENV_PREFIX: &str = "VOX_";

/// CLI flag overrides; `None` leaves the resolved value untouched.
#[derive(Debug, Clone, Default)]
pub struct FlagOverrides {
    /// `--provider` value.
    pub provider: Option<String>,
    /// `--model` value.
    pub model: Option<String>,
    /// `--base-url` value.
    pub base_url: Option<String>,
    /// `--api-key-env` value.
    pub api_key_env: Option<String>,
    /// `--log` value.
    pub log: Option<String>,
}

/// Inputs to configuration resolution.
#[derive(Debug, Clone, Default)]
pub struct LoadInputs {
    /// Optional user configuration file.
    pub user_config: Option<PathBuf>,
    /// Optional project configuration file.
    pub project_config: Option<PathBuf>,
    /// Environment map.
    pub env: HashMap<String, String>,
    /// CLI flag overrides (highest precedence).
    pub flags: FlagOverrides,
}

/// Load, layer, and validate configuration.
///
/// Precedence, lowest first: compiled defaults, user file, project file,
/// `VOX_*` environment variables, CLI flags. Any validation failure returns
/// [`VoxError::Config`] with a generic user-facing message and a detail
/// string for the log.
pub fn load(inputs: &LoadInputs) -> VoxResult<Config> {
    let config = load_layers(inputs)?;
    validate(&config)?;
    Ok(config)
}

/// Discover the project configuration file at or above `start`.
///
/// The walk stops at the repository boundary: a directory containing `.git`
/// is the last one examined, so a parent repository never leaks its settings
/// into a nested project.
pub fn discover_project_config(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let candidate = dir.join(PROJECT_CONFIG_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if dir.join(".git").exists() {
            return None;
        }
    }
    None
}

/// Merge configuration layers without validation.
///
/// Files merge field by field (see [`crate::files`]); environment and flags
/// then override individual fields.
fn load_layers(inputs: &LoadInputs) -> VoxResult<Config> {
    let mut config = merge_files(&[
        inputs.user_config.as_deref(),
        inputs.project_config.as_deref(),
    ])?
    .unwrap_or_default();
    apply_env(&mut config, &inputs.env);
    apply_flags(&mut config, &inputs.flags);
    Ok(config)
}

/// Apply `VOX_*` environment overrides.
fn apply_env(config: &mut Config, env: &HashMap<String, String>) {
    let take = |key: &str| env.get(&format!("{ENV_PREFIX}{key}")).cloned();
    if let Some(value) = take("PROVIDER") {
        config.general.provider = value;
    }
    if let Some(value) = take("MODEL") {
        config.general.model = value;
    }
    if let Some(value) = take("BASE_URL") {
        config.general.base_url = Some(value);
    }
    if let Some(value) = take("API_KEY_ENV") {
        config.general.api_key_env = value;
    }
    if let Some(value) = take("LOG") {
        config.general.log = value;
    }
}

/// Apply CLI flag overrides.
fn apply_flags(config: &mut Config, flags: &FlagOverrides) {
    if let Some(value) = &flags.provider {
        config.general.provider = value.clone();
    }
    if let Some(value) = &flags.model {
        config.general.model = value.clone();
    }
    if let Some(value) = &flags.base_url {
        config.general.base_url = Some(value.clone());
    }
    if let Some(value) = &flags.api_key_env {
        config.general.api_key_env = value.clone();
    }
    if let Some(value) = &flags.log {
        config.general.log = value.clone();
    }
}
