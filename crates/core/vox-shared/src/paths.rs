//! Platform resolution of the Vox data and configuration directories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Root directory for Vox user data (sessions, logs).
///
/// Resolution order: `VOX_DATA_DIR`, then the platform default. A relative
/// `VOX_DATA_DIR` is resolved against the current working directory.
pub fn data_dir() -> PathBuf {
    data_dir_from(&env_map())
}

/// Root directory for the Vox user configuration file.
///
/// Resolution order: `VOX_CONFIG_DIR`, then the platform configuration
/// directory.
pub fn config_dir() -> PathBuf {
    let env = env_map();
    if let Some(dir) = env.get("VOX_CONFIG_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    platform_config_dir(&env)
}

/// Pure data-directory resolution over an explicit environment map.
pub fn data_dir_from(env: &HashMap<String, String>) -> PathBuf {
    if let Some(dir) = env.get("VOX_DATA_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    platform_data_dir(env)
}

/// Read the process environment into a map.
fn env_map() -> HashMap<String, String> {
    std::env::vars().collect()
}

/// Platform default data directory.
fn platform_data_dir(env: &HashMap<String, String>) -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(base) = env.get("APPDATA") {
            return Path::new(base).join("vox");
        }
    }
    if cfg!(target_os = "macos") {
        if let Some(home) = env.get("HOME") {
            return Path::new(home).join("Library/Application Support/vox");
        }
    }
    if let Some(xdg) = env.get("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return Path::new(xdg).join("vox");
        }
    }
    let home = env.get("HOME").cloned().unwrap_or_default();
    Path::new(&home).join(".local/share/vox")
}

/// Platform default configuration directory.
fn platform_config_dir(env: &HashMap<String, String>) -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(base) = env.get("APPDATA") {
            return Path::new(base).join("vox");
        }
    }
    if cfg!(target_os = "macos") {
        if let Some(home) = env.get("HOME") {
            return Path::new(home).join("Library/Application Support/vox");
        }
    }
    let home = env.get("HOME").cloned().unwrap_or_default();
    Path::new(&home).join(".config/vox")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn explicit_data_dir_wins() {
        let env = env_with(&[("VOX_DATA_DIR", "/tmp/vox-data"), ("HOME", "/home/u")]);
        assert_eq!(data_dir_from(&env), PathBuf::from("/tmp/vox-data"));
    }

    #[test]
    fn empty_data_dir_falls_back_to_platform_default() {
        let env = env_with(&[("VOX_DATA_DIR", ""), ("HOME", "/home/u")]);
        let resolved = data_dir_from(&env);
        assert!(resolved.starts_with("/home/u"), "got {resolved:?}");
    }

    #[test]
    fn xdg_data_home_is_used_when_set() {
        let env = env_with(&[("XDG_DATA_HOME", "/data"), ("HOME", "/home/u")]);
        assert_eq!(data_dir_from(&env), PathBuf::from("/data/vox"));
    }
}
