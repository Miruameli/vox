//! Integration tests for project configuration discovery.

use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};

use vox_config::{discover_project_config, load, LoadInputs, PROJECT_CONFIG_NAME};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a unique scratch directory for one test.
fn scratch_dir() -> std::path::PathBuf {
    let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("vox-disc-test-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).expect("create dir");
    dir
}

/// Create a repository with a nested package directory.
fn repo_with_nested(root: &std::path::Path) -> std::path::PathBuf {
    let nested = root.join("packages/app");
    fs::create_dir_all(&nested).expect("create dirs");
    fs::create_dir_all(root.join(".git")).expect("create git dir");
    nested
}

#[test]
fn discovery_stops_at_repository_boundary() {
    let dir = scratch_dir();
    let repo = dir.join("repo");
    let nested = repo_with_nested(&repo);
    fs::write(repo.join(PROJECT_CONFIG_NAME), "model = \"repo-model\"\n").expect("write");
    let found = discover_project_config(&nested).expect("found");
    assert_eq!(found, repo.join(PROJECT_CONFIG_NAME));
}

#[test]
fn discovery_does_not_cross_into_unrelated_parent() {
    let dir = scratch_dir();
    let repo = dir.join("repo");
    let nested = repo_with_nested(&repo);
    fs::write(dir.join(PROJECT_CONFIG_NAME), "model = \"parent-model\"\n").expect("write");
    assert!(discover_project_config(&nested).is_none());
}

#[test]
fn env_map_can_be_passed_without_process_environment() {
    let mut map = std::collections::HashMap::new();
    map.insert("VOX_MODEL".to_string(), "map-model".to_string());
    let config = load(&LoadInputs {
        env: map,
        ..LoadInputs::default()
    })
    .expect("load");
    assert_eq!(config.general.model, "map-model");
}
