//! Grep and glob tool behavior: matching, caps, and path safety.

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use vox_domain::ToolCall;
use vox_engine_tools::{ToolHarness, GLOB_PATH_LIMIT, GREP_MATCH_LIMIT};

fn call(name: &str, arguments: serde_json::Value) -> ToolCall {
    ToolCall {
        id: "t1".to_string(),
        name: name.to_string(),
        arguments,
    }
}

/// Run one tool against a seeded project tree.
fn run(name: &str, arguments: serde_json::Value) -> (TempDir, String) {
    let dir = TempDir::new().expect("temp dir");
    seed(dir.path());
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness.call(&call(name, arguments)).expect("tool call");
    assert!(!result.is_error, "{}", result.content);
    (dir, result.content)
}

/// Nested sources sharing a token, plus noise the walker skips:
/// a dotfile and a build-output directory.
fn seed(root: &std::path::Path) {
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
    assert!(!out.contains(".hidden.rs"), "{out}");
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
}

#[test]
fn grep_rejects_paths_outside_the_root() {
    let dir = TempDir::new().expect("temp dir");
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
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
fn glob_matches_star_within_a_segment() {
    let (_dir, out) = run("glob", json!({ "pattern": "src/*.rs" }));
    assert_eq!(out.lines().collect::<Vec<_>>(), vec!["src/alpha.rs"]);
}

#[test]
fn glob_double_star_spans_directories() {
    let (_dir, out) = run("glob", json!({ "pattern": "**/*.rs" }));
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"src/alpha.rs"), "{out}");
    assert!(lines.contains(&"src/nested/beta.rs"), "{out}");
    assert!(!lines.contains(&"target/out.rs"), "{out}");
}

#[test]
fn glob_single_star_does_not_cross_directories() {
    let (_dir, out) = run("glob", json!({ "pattern": "*/*.rs" }));
    assert_eq!(out.lines().collect::<Vec<_>>(), vec!["src/alpha.rs"]);
}

#[test]
fn glob_caps_output_and_announces_the_remainder() {
    let dir = TempDir::new().expect("temp dir");
    for i in 0..(GLOB_PATH_LIMIT + 3) {
        fs::write(dir.path().join(format!("f{i}.rs")), "").expect("file");
    }
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness
        .call(&call("glob", json!({ "pattern": "*.rs" })))
        .expect("glob");
    let lines: Vec<&str> = result.content.lines().collect();
    assert_eq!(lines.len(), GLOB_PATH_LIMIT + 1, "{}", lines.len());
    assert_eq!(lines[GLOB_PATH_LIMIT], "... 3 more paths");
}

#[test]
fn unknown_tools_stay_failures() {
    let dir = TempDir::new().expect("temp dir");
    let mut harness =
        ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let result = harness.call(&call("sed", json!({}))).expect("unknown tool");
    assert!(result.is_error);
    assert!(result.content.contains("unknown tool"));
}
