//! Edit, write, and dispatch round trips: anchor-verified patches,
//! atomic whole-file writes with journaling, and refusal of unknown
//! tools and escaping paths.

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use vox_domain::ToolCall;
use vox_engine_tools::ToolHarness;

fn call(name: &str, id: &str, arguments: serde_json::Value) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments,
    }
}

fn project(content: &str) -> (TempDir, ToolHarness) {
    let dir = TempDir::new().expect("temp dir");
    fs::write(dir.path().join("main.rs"), content).expect("seed");
    let harness = ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    (dir, harness)
}

fn hash_of(line: &str) -> String {
    vox_engine_edit::line_hash(line)
}

#[test]
fn edit_applies_a_hashed_patch_from_a_tool_call() {
    let (dir, mut harness) = project("alpha\nbeta\ngamma\n");
    let result = harness
        .call(&call(
            "edit",
            "c1",
            json!({
                "path": "main.rs",
                "ops": [{
                    "op": "replace",
                    "line": 2,
                    "expect_hash": hash_of("beta"),
                    "new_lines": ["BETA"],
                }],
            }),
        ))
        .expect("dispatch");
    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("+1 -1"));
    assert_eq!(
        fs::read_to_string(dir.path().join("main.rs")).expect("read"),
        "alpha\nBETA\ngamma\n"
    );
    let records = harness.undo_records().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].pre_image.as_deref(),
        Some("alpha\nbeta\ngamma\n")
    );
}

#[test]
fn edit_with_stale_anchor_fails_and_leaves_the_file_intact() {
    let (dir, mut harness) = project("alpha\nbeta\n");
    let result = harness
        .call(&call(
            "edit",
            "c1",
            json!({
                "path": "main.rs",
                "ops": [{
                    "op": "delete",
                    "line": 1,
                    "expect_hash": hash_of("not-alpha"),
                }],
            }),
        ))
        .expect("dispatch");
    assert!(result.is_error);
    assert_eq!(
        fs::read_to_string(dir.path().join("main.rs")).expect("read"),
        "alpha\nbeta\n"
    );
    assert!(harness.undo_records().expect("records").is_empty());
}

#[test]
fn edit_with_malformed_arguments_returns_a_tool_failure() {
    let (_dir, mut harness) = project("x\n");
    let result = harness
        .call(&call("edit", "c1", json!({"path": "main.rs"})))
        .expect("dispatch");
    assert!(result.is_error);
    assert!(result.content.contains("ops"), "{}", result.content);
}

#[test]
fn write_creates_and_replaces_with_a_journal() {
    let (dir, mut harness) = project("old\n");
    let created = harness
        .call(&call(
            "write",
            "c1",
            json!({"path": "new.txt", "content": "brand new\n"}),
        ))
        .expect("dispatch");
    assert!(!created.is_error);
    assert!(created.content.starts_with("created"));
    assert_eq!(
        fs::read_to_string(dir.path().join("new.txt")).expect("read"),
        "brand new\n"
    );

    let replaced = harness
        .call(&call(
            "write",
            "c2",
            json!({"path": "main.rs", "content": "fresh\n"}),
        ))
        .expect("dispatch");
    assert!(replaced.content.starts_with("replaced"));
    let records = harness.undo_records().expect("records");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].pre_image, None);
    assert_eq!(records[1].pre_image.as_deref(), Some("old\n"));
}

#[test]
fn unknown_tool_and_traversal_are_refused() {
    let (dir, mut harness) = project("x\n");
    let unknown = harness
        .call(&call("delete", "c1", json!({"path": "main.rs"})))
        .expect("dispatch");
    assert!(unknown.is_error);
    assert!(unknown.content.contains("unknown tool"));

    let escape = harness
        .call(&call(
            "write",
            "c2",
            json!({"path": "../escape.txt", "content": "x"}),
        ))
        .expect("dispatch");
    assert!(escape.is_error);
    assert!(!dir
        .path()
        .parent()
        .expect("parent")
        .join("escape.txt")
        .exists());
}
