//! Configuration file parsing and field-level layer merging.

use std::path::Path;

use toml::Value;
use vox_shared::redaction::contains_secret;
use vox_shared::{VoxError, VoxResult};

use crate::schema::Config;

/// Parse one configuration file into raw TOML, rejecting embedded secrets.
///
/// The secret check runs on the raw text before parsing so a credential can
/// never reach the merged document, even transiently.
pub fn parse_file(path: &Path) -> VoxResult<Value> {
    let raw = std::fs::read_to_string(path)?;
    if contains_secret(&raw) {
        return Err(VoxError::Config {
            detail: format!("credential-shaped value in {}", path.display()),
        });
    }
    toml::from_str::<Value>(&raw).map_err(|error| VoxError::Config {
        detail: format!("{}: {error}", path.display()),
    })
}

/// Merge configuration files, lowest priority first, then validate.
///
/// Files merge field by field: a project file that sets only
/// `general.model` keeps the user file's other settings. The merged
/// document is deserialized once, so unknown keys still fail validation.
pub fn merge_files(paths: &[Option<&Path>]) -> VoxResult<Option<Config>> {
    let mut merged: Option<Value> = None;
    for path in paths.iter().flatten() {
        let value = parse_file(path)?;
        merged = Some(match merged {
            None => value,
            Some(base) => deep_merge(base, value),
        });
    }
    merged
        .map(|value| {
            value.try_into().map_err(|error| VoxError::Config {
                detail: format!("merged configuration invalid: {error}"),
            })
        })
        .transpose()
}

/// Deep-merge two TOML values; `overlay` wins on conflicting leaves.
fn deep_merge(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Table(mut base), Value::Table(overlay)) => {
            for (key, value) in overlay {
                let merged = match base.remove(&key) {
                    Some(existing) => deep_merge(existing, value),
                    None => value,
                };
                base.insert(key, merged);
            }
            Value::Table(base)
        }
        (_, overlay) => overlay,
    }
}
