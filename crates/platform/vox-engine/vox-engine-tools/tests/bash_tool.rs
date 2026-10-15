//! Bash tool behavior: argv execution, working directory, timeout,
//! exit codes, and output caps (ADR-0011).

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use vox_domain::ToolCall;
use vox_engine_tools::{ToolHarness, BASH_OUTPUT_LIMIT};

/// A tool call with a fixed id.
fn call(name: &str, arguments: serde_json::Value) -> ToolCall {
    ToolCall {
        id: "t1".to_string(),
        name: name.to_string(),
        arguments,
    }
}

/// A harness over a fresh empty project directory.
fn harness() -> (TempDir, ToolHarness) {
    let dir = TempDir::new().expect("temp dir");
    let harness = ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    (dir, harness)
}

#[test]
fn bash_runs_argv_without_a_shell() {
    // The argv form passes each element as one program argument:
    // no quoting rules, and a file relative to the project root is
    // found because the command runs there.
    let (dir, mut harness) = harness();
    fs::write(dir.path().join("greeting.txt"), "hello vox\n").expect("seed");
    let result = harness
        .call(&call("bash", json!({ "command": ["cat", "greeting.txt"] })))
        .expect("bash");
    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("exit 0"), "{}", result.content);
    assert!(result.content.contains("hello vox"), "{}", result.content);
}

#[test]
fn bash_reports_a_nonzero_exit_as_its_answer() {
    // A failing command is the model's information, not a tool
    // failure: the answer carries the exit code so the model can
    // read what went wrong.
    let (_dir, mut harness) = harness();
    let result = harness
        .call(&call("bash", json!({ "command": ["sh", "-c", "exit 7"] })))
        .expect("bash");
    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("exit 7"), "{}", result.content);
}

#[test]
fn bash_times_out_and_says_so() {
    let (_dir, mut harness) = harness();
    let result = harness
        .call(&call(
            "bash",
            json!({ "command": ["sleep", "5"], "timeout_ms": 100 }),
        ))
        .expect("bash");
    assert!(result.is_error, "{}", result.content);
    assert!(
        result.content.contains("timed out after 100 ms"),
        "{}",
        result.content
    );
}

#[test]
fn bash_caps_output_and_announces_the_cut() {
    let (_dir, mut harness) = harness();
    let result = harness
        .call(&call(
            "bash",
            json!({ "command": ["sh", "-c", "seq 1 5000"] }),
        ))
        .expect("bash");
    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("bytes truncated"),
        "{}",
        result.content
    );
    // The answer keeps stdout under the cap plus the markers around
    // it; the raw stream never reaches the conversation whole.
    let stdout_len = result
        .content
        .split("stdout:\n")
        .nth(1)
        .map(|rest| rest.split("\n... ").next().unwrap_or(rest).len())
        .unwrap_or(0);
    assert!(stdout_len <= BASH_OUTPUT_LIMIT, "stdout {stdout_len}");
}

#[test]
fn bash_rejects_an_empty_command() {
    let (_dir, mut harness) = harness();
    let result = harness
        .call(&call("bash", json!({ "command": [] })))
        .expect("bash");
    assert!(result.is_error, "{}", result.content);
    assert!(
        result.content.contains("must not be empty"),
        "{}",
        result.content
    );
}

#[test]
fn bash_survives_a_command_that_leaves_a_background_process() {
    // Regression: `sh -c "sleep 30 & ..."` exits while the
    // background process still holds the pipe write end. Joining
    // the pipe readers on the normal exit path hung the session
    // forever. The call must return within its timeout and say the
    // captured output may be partial.
    let (_dir, mut harness) = harness();
    let result = harness
        .call(&call(
            "bash",
            json!({ "command": ["sh", "-c", "sleep 30 & echo started"], "timeout_ms": 1000 }),
        ))
        .expect("bash");
    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("exit 0"), "{}", result.content);
    assert!(result.content.contains("started"), "{}", result.content);
    assert!(
        result.content.contains("output may be incomplete"),
        "{}",
        result.content
    );
}

#[test]
fn bash_reports_a_command_it_cannot_start() {
    let (_dir, mut harness) = harness();
    let result = harness
        .call(&call(
            "bash",
            json!({ "command": ["definitely-not-a-binary"] }),
        ))
        .expect("bash");
    assert!(result.is_error, "{}", result.content);
    assert!(result.content.contains("cannot run"), "{}", result.content);
}
