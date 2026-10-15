//! Glob tool behavior: path patterns, caps, and argument validation.

mod fixtures;

use serde_json::json;
use vox_engine_tools::GLOB_PATH_LIMIT;

use fixtures::{call, empty_harness, run};

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
    let dir = tempfile::TempDir::new().expect("temp dir");
    for i in 0..(GLOB_PATH_LIMIT + 3) {
        std::fs::write(dir.path().join(format!("f{i}.rs")), "").expect("file");
    }
    let mut harness =
        vox_engine_tools::ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl"))
            .expect("harness");
    let result = harness
        .call(&call("glob", json!({ "pattern": "*.rs" })))
        .expect("glob");
    let lines: Vec<&str> = result.content.lines().collect();
    assert_eq!(lines.len(), GLOB_PATH_LIMIT + 1, "{}", lines.len());
    assert_eq!(lines[GLOB_PATH_LIMIT], "... 3 more paths");
}

#[test]
fn glob_returns_paths_in_sorted_order() {
    // File creation order does not match sorted order, and the walk
    // order is unspecified: the answer's stability must come from
    // the tool itself.
    let dir = tempfile::TempDir::new().expect("temp dir");
    for name in ["zeta.rs", "alpha.rs", "mid.rs"] {
        std::fs::write(dir.path().join(name), "").expect("file");
    }
    let mut harness =
        vox_engine_tools::ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl"))
            .expect("harness");
    let result = harness
        .call(&call("glob", json!({ "pattern": "*.rs" })))
        .expect("glob");
    assert_eq!(
        result.content.lines().collect::<Vec<_>>(),
        vec!["alpha.rs", "mid.rs", "zeta.rs"],
        "{}",
        result.content
    );
}

#[test]
fn glob_rejects_empty_pattern_like_grep() {
    let (_dir, mut harness) = empty_harness();
    let result = harness
        .call(&call("glob", json!({ "pattern": "" })))
        .expect("empty pattern");
    assert!(result.is_error, "{}", result.content);
    assert!(
        result.content.contains("must not be empty"),
        "{}",
        result.content
    );
}
