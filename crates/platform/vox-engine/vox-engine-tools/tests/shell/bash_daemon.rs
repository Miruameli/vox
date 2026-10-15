//! Bash tool and descendants that outlive the command (ADR-0011).
//!
//! A command may end while a child of it still holds the pipe write
//! end. Both directions matter: the session must not hang, and a
//! finished command's answer must not wait for its whole budget.

use serde_json::json;

use crate::fixtures::{call, empty_harness};

#[test]
fn bash_survives_a_command_that_leaves_a_background_process() {
    // Regression: `sh -c "sleep 30 & ..."` exits while the
    // background process still holds the pipe write end. Joining
    // the pipe readers on the normal exit path hung the session
    // forever. The call must return within its timeout and say the
    // captured output may be partial.
    let (_dir, mut harness) = empty_harness();
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
fn bash_returns_promptly_when_a_command_leaves_a_daemon() {
    // The command's answer is ready the moment it exits; a
    // descendant holding the write end must not stretch the call
    // out to the budget. Regression for the exit path collecting
    // until the full deadline (#33).
    let (_dir, mut harness) = empty_harness();
    let started = std::time::Instant::now();
    let result = harness
        .call(&call(
            "bash",
            json!({ "command": ["sh", "-c", "echo ready; sleep 30 &"], "timeout_ms": 30000 }),
        ))
        .expect("bash");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(2),
        "took {:?}",
        started.elapsed()
    );
    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("exit 0"), "{}", result.content);
    assert!(result.content.contains("ready"), "{}", result.content);
    assert!(
        result.content.contains("output may be incomplete"),
        "{}",
        result.content
    );
}
