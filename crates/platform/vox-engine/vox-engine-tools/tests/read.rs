//! Tool-call round trips over the harness: read paging, edit with
//! anchor verification, atomic writes, and the parity of the read view
//! with the engine's whole-file hashed view.

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use vox_domain::ToolCall;
use vox_engine_edit::hashed_view;
use vox_engine_tools::{hashed_view_range, ToolHarness};

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
fn read_returns_hashed_view_with_absolute_numbers() {
    let (_dir, mut harness) = project("alpha\nbeta\ngamma\n");
    let result = harness
        .call(&call("read", "c1", json!({"path": "main.rs"})))
        .expect("dispatch");
    assert!(!result.is_error);
    let view = result.content;
    let first = view.lines().next().expect("first line");
    let (number, rest) = first.split_once('|').expect("separator");
    assert_eq!(number.trim(), "1");
    let (hash, content) = rest.split_once('|').expect("hash");
    assert_eq!(content, "alpha");
    assert_eq!(hash, hash_of("alpha"));
}

#[test]
fn read_pages_with_offset_and_limit() {
    let (_dir, mut harness) = project("l1\nl2\nl3\nl4\nl5\n");
    let result = harness
        .call(&call(
            "read",
            "c1",
            json!({"path": "main.rs", "offset": 2, "limit": 2}),
        ))
        .expect("dispatch");
    assert!(!result.is_error);
    let lines: Vec<&str> = result.content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].trim_start().starts_with("2|"));
    assert!(lines[1].trim_start().starts_with("3|"));
}

#[test]
fn read_out_of_range_offset_fails_without_touching_the_file() {
    let (dir, mut harness) = project("only\n");
    let result = harness
        .call(&call("read", "c1", json!({"path": "main.rs", "offset": 9})))
        .expect("dispatch");
    assert!(result.is_error);
    assert_eq!(
        fs::read_to_string(dir.path().join("main.rs")).expect("read"),
        "only\n"
    );
}

#[test]
fn read_rejects_oversized_pages_before_reading() {
    let (_dir, mut harness) = project("x\n");
    let result = harness
        .call(&call(
            "read",
            "c1",
            json!({"path": "main.rs", "limit": 5000}),
        ))
        .expect("dispatch");
    assert!(result.is_error);
    assert!(result.content.contains("page cap"));
}

#[test]
fn read_view_range_matches_the_engine_whole_file_view() {
    let file = "one\ntwo\nthree\n";
    assert_eq!(
        hashed_view_range(file, 1, usize::MAX).expect("range"),
        hashed_view(file)
    );
}

#[test]
fn empty_file_reads_as_an_empty_view() {
    let dir = TempDir::new().expect("temp dir");
    fs::write(dir.path().join("empty.txt"), "").expect("seed");
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness
        .call(&call("read", "c1", json!({"path": "empty.txt"})))
        .expect("run");
    assert_eq!(result.content, "");
}
