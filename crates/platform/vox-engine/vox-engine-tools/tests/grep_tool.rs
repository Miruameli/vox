//! Grep tool behavior: matching, ordering, caps, and path safety.

mod fixtures;

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use vox_engine_tools::{ToolHarness, GREP_MATCH_LIMIT};

use fixtures::{call, failing_harness, run};

#[test]
fn grep_finds_every_match_with_line_numbers() {
    let (_dir, out) = run("grep", json!({ "pattern": "needle" }));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert_eq!(lines[0], "src/alpha.rs:1:fn needle() {}");
    assert_eq!(lines[1], "src/nested/beta.rs:1:// needle here");
}

#[test]
fn grep_skips_dotfiles_and_build_output() {
    let (_dir, out) = run("grep", json!({ "pattern": "needle" }));
    assert!(!out.contains("hidden"), "{out}");
    assert!(!out.contains("target"), "{out}");
}

#[test]
fn grep_caps_output_and_announces_the_remainder() {
    let dir = TempDir::new().expect("temp dir");
    let mut text = String::new();
    for i in 0..(GREP_MATCH_LIMIT + 5) {
        text.push_str(&format!("line {i} needle\n"));
    }
    fs::write(dir.path().join("big.rs"), &text).expect("big");
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness
        .call(&call("grep", json!({ "pattern": "needle" })))
        .expect("grep");
    let lines: Vec<&str> = result.content.lines().collect();
    assert_eq!(lines.len(), GREP_MATCH_LIMIT + 1, "{}", lines.len());
    assert_eq!(lines[GREP_MATCH_LIMIT], "... 5 more matches");
    // The cap must keep the first matches in reading order: a
    // lexicographic sort would show lines 1, 10, 100, 101, ... and
    // cut the true first matches instead.
    assert_eq!(lines[0], "big.rs:1:line 0 needle");
    assert_eq!(lines[1], "big.rs:2:line 1 needle");
    assert_eq!(lines[2], "big.rs:3:line 2 needle");
}

#[test]
fn grep_orders_lines_numerically_within_a_file() {
    let dir = TempDir::new().expect("temp dir");
    let mut text = String::new();
    for i in 0..120 {
        let line = if i == 1 || i == 9 || i == 99 {
            format!("match {i}\n")
        } else {
            format!("filler {i}\n")
        };
        text.push_str(&line);
    }
    fs::write(dir.path().join("numbers.rs"), &text).expect("numbers");
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness
        .call(&call("grep", json!({ "pattern": "match " })))
        .expect("grep");
    let lines: Vec<&str> = result.content.lines().collect();
    assert_eq!(
        lines,
        vec![
            "numbers.rs:2:match 1",
            "numbers.rs:10:match 9",
            "numbers.rs:100:match 99",
        ],
        "{lines:?}"
    );
}

#[test]
fn grep_rejects_paths_outside_the_root() {
    let (_dir, mut harness) = failing_harness();
    let result = harness
        .call(&call(
            "grep",
            json!({ "pattern": "x", "path": "../escape" }),
        ))
        .expect("escape attempt");
    assert!(result.is_error, "{}", result.content);
    assert!(result.content.contains("escapes"), "{}", result.content);
}

#[test]
fn unknown_tools_stay_failures() {
    let (_dir, mut harness) = failing_harness();
    let result = harness.call(&call("sed", json!({}))).expect("unknown tool");
    assert!(result.is_error);
    assert!(result.content.contains("unknown tool"));
}
