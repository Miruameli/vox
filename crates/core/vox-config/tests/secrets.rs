//! Configuration files must never carry credential-shaped values,
//! whether the secret sits in an assignment or in a bare literal.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use vox_config::{load, LoadInputs};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a unique scratch directory for one test.
fn scratch_dir() -> PathBuf {
    let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("vox-secrets-test-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).expect("create dir");
    dir
}

#[test]
fn bare_literal_keys_are_rejected() {
    let path = scratch_dir().join("bare.toml");
    // Assembled from parts so this source carries no credential-shaped
    // literal for secret scanners to flag; the file the loader reads
    // holds the real shape.
    let literal = ["sk-", "abcdef12345678"].concat();
    fs::write(&path, format!("model = \"m\"\nnote = \"{literal}\"\n")).expect("write file");
    let error = load(&LoadInputs {
        project_config: Some(path),
        ..LoadInputs::default()
    })
    .expect_err("reject");
    assert!(error.detail().contains("credential"));
}

#[test]
fn short_literals_stay_allowed() {
    let path = scratch_dir().join("short.toml");
    fs::write(&path, "model = \"m\"\nnote = \"sk-short\"\n").expect("write file");
    let error = load(&LoadInputs {
        project_config: Some(path),
        ..LoadInputs::default()
    });
    // "sk-short" is shorter than the literal-key threshold; it fails on
    // the unknown field, proving the credential scan did not fire.
    let detail = error.expect_err("unknown field").detail();
    assert!(detail.contains("unknown field"), "got: {detail}");
}
