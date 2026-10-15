//! Integration tests for field-level configuration file merging.

use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};

use vox_config::{load, LoadInputs};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a unique scratch directory for one test.
fn scratch_dir() -> std::path::PathBuf {
    let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("vox-merge-test-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).expect("create dir");
    dir
}

/// Write a file with the given contents.
fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write file");
}

#[test]
fn project_file_overrides_only_its_own_fields() {
    let dir = scratch_dir();
    let user = dir.join("user.toml");
    let project = dir.join("project.toml");
    write_file(
        &user,
        "[general]\nprovider = \"openai\"\nmodel = \"user-model\"\nlog = \"debug\"\n",
    );
    write_file(&project, "[general]\nmodel = \"project-model\"\n");
    let config = load(&LoadInputs {
        user_config: Some(user),
        project_config: Some(project),
        ..LoadInputs::default()
    })
    .expect("load");
    assert_eq!(config.general.model, "project-model");
    // Fields the project file never mentions keep the user layer's values.
    assert_eq!(config.general.provider, "openai");
    assert_eq!(config.general.log, "debug");
}

#[test]
fn unknown_keys_are_rejected_after_merge() {
    let dir = scratch_dir();
    let user = dir.join("user.toml");
    let project = dir.join("project.toml");
    write_file(&user, "[general]\nmodel = \"user-model\"\n");
    write_file(&project, "[general]\nnot_a_key = 1\n");
    let error = load(&LoadInputs {
        user_config: Some(user),
        project_config: Some(project),
        ..LoadInputs::default()
    })
    .expect_err("reject");
    assert!(error.detail().contains("unknown field"));
}

#[test]
fn project_file_alone_is_enough() {
    let dir = scratch_dir();
    let project = dir.join("project.toml");
    write_file(&project, "[general]\nmodel = \"solo-model\"\n");
    let config = load(&LoadInputs {
        project_config: Some(project),
        ..LoadInputs::default()
    })
    .expect("load");
    assert_eq!(config.general.model, "solo-model");
    assert_eq!(config.general.provider, "openai");
}
