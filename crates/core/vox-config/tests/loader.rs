//! Integration tests for layered configuration loading.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use vox_config::{load, FlagOverrides, LoadInputs};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a unique scratch directory for one test.
fn scratch_dir() -> PathBuf {
    let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("vox-config-test-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).expect("create dir");
    dir
}

/// Write a file with the given contents.
fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write file");
}

/// Build inputs with only an environment map and flags.
fn inputs(env: &[(&str, &str)], flags: FlagOverrides) -> LoadInputs {
    LoadInputs {
        env: env
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
        flags,
        ..LoadInputs::default()
    }
}

#[test]
fn defaults_when_nothing_is_configured() {
    let config = load(&inputs(&[], FlagOverrides::default())).expect("load");
    assert_eq!(config.general.provider, "openai");
    assert_eq!(config.general.model, "gpt-4o-mini");
    assert_eq!(config.general.api_key_env, "OPENAI_API_KEY");
    assert!(config.general.base_url.is_none());
}

#[test]
fn environment_overrides_defaults() {
    let config = load(&inputs(
        &[
            ("VOX_PROVIDER", "openai"),
            ("VOX_MODEL", "gpt-4o-mini-2026"),
            ("VOX_LOG", "debug"),
        ],
        FlagOverrides::default(),
    ))
    .expect("load");
    assert_eq!(config.general.model, "gpt-4o-mini-2026");
    assert_eq!(config.general.log, "debug");
}

#[test]
fn flags_override_environment() {
    let config = load(&inputs(
        &[("VOX_MODEL", "from-env")],
        FlagOverrides {
            model: Some("from-flag".into()),
            ..FlagOverrides::default()
        },
    ))
    .expect("load");
    assert_eq!(config.general.model, "from-flag");
}

#[test]
fn unknown_keys_are_rejected() {
    let dir = scratch_dir();
    let path = dir.join("bad.toml");
    write_file(&path, "not_a_key = 1\n");
    let error = load(&LoadInputs {
        project_config: Some(path),
        ..LoadInputs::default()
    })
    .expect_err("reject");
    assert!(error.detail().contains("unknown field"));
}

#[test]
fn credential_shaped_values_are_rejected() {
    // Built at runtime: the repository must never carry a literal that
    // a secret scanner flags, while the assignment-shape rejection
    // stays exercised with a key-shaped value.
    let key = ["sk-", "abcdef12345678"].concat();
    let dir = scratch_dir();
    let path = dir.join("leaky.toml");
    write_file(
        &path,
        &format!("model = \"gpt-4o-mini\"\napi_key = \"{key}\"\n"),
    );
    let error = load(&LoadInputs {
        project_config: Some(path),
        ..LoadInputs::default()
    })
    .expect_err("reject");
    assert!(error.detail().contains("credential"));
}

#[test]
fn http_base_url_is_rejected_outside_loopback() {
    let config_error = load(&inputs(
        &[("VOX_BASE_URL", "http://example.com/v1")],
        FlagOverrides::default(),
    ))
    .expect_err("reject");
    assert!(config_error.detail().contains("base_url"));
}

#[test]
fn http_base_url_is_allowed_on_loopback() {
    let config = load(&inputs(
        &[("VOX_BASE_URL", "http://127.0.0.1:11434/v1")],
        FlagOverrides::default(),
    ))
    .expect("load");
    assert_eq!(
        config.general.base_url.as_deref(),
        Some("http://127.0.0.1:11434/v1")
    );
}

#[test]
fn unknown_log_filter_is_rejected() {
    let error =
        load(&inputs(&[("VOX_LOG", "chatty")], FlagOverrides::default())).expect_err("reject");
    assert!(error.detail().contains("log filter"));
}
