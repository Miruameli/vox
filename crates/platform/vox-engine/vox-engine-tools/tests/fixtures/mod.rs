//! Shared helpers for the search tool tests: one project tree with
//! nested sources sharing a token, plus noise the walker skips (a
//! dotfile and a build-output directory).

use std::fs;

use serde_json::Value;
use tempfile::TempDir;
use vox_domain::ToolCall;
use vox_engine_tools::ToolHarness;

/// A tool call with a fixed id.
pub fn call(name: &str, arguments: Value) -> ToolCall {
    ToolCall {
        id: "t1".to_string(),
        name: name.to_string(),
        arguments,
    }
}

/// Run one tool against the seeded project tree.
pub fn run(name: &str, arguments: Value) -> (TempDir, String) {
    let dir = TempDir::new().expect("temp dir");
    seed(dir.path());
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness.call(&call(name, arguments)).expect("tool call");
    assert!(!result.is_error, "{}", result.content);
    (dir, result.content)
}

/// A fresh harness over an empty directory, for calls that must fail.
pub fn failing_harness() -> (TempDir, ToolHarness) {
    let dir = TempDir::new().expect("temp dir");
    let harness = ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    (dir, harness)
}

/// Seed the project tree the search answers are checked against.
pub fn seed(root: &std::path::Path) {
    fs::create_dir_all(root.join("src/nested")).expect("dirs");
    fs::create_dir_all(root.join("target")).expect("dirs");
    fs::write(root.join("src/alpha.rs"), "fn needle() {}\nfn plain() {}\n").expect("alpha");
    fs::write(
        root.join("src/nested/beta.rs"),
        "// needle here\nfn other() {}\n",
    )
    .expect("beta");
    fs::write(root.join(".hidden.rs"), "needle\n").expect("hidden");
    fs::write(root.join("target/out.rs"), "needle\n").expect("out");
}
