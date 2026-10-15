//! Loop behaviour: one full tool cycle, the stop path, and the turn and
//! tool-error bounds, with a scripted mock provider.

mod fixtures;

use std::fs;

use vox_agent::{FinishReason, LoopEvent};
use vox_session::replay;

use fixtures::{agent, project, read_call, stop_turn, tool_call_turn};

#[tokio::test]
async fn one_full_tool_cycle_then_stop() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[read_call()]), stop_turn("done")],
    );
    let mut events = Vec::new();
    let reason = agent
        .run("read the file", &mut |e| events.push(e))
        .await
        .expect("run");

    assert_eq!(reason, FinishReason::ModelStopped);
    assert!(events.contains(&LoopEvent::ToolStarted {
        name: "read".into()
    }));
    assert!(events.contains(&LoopEvent::ToolFinished {
        name: "read".into(),
        is_error: false,
    }));
    let texts: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            LoopEvent::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(texts.iter().any(|t| t.contains("done")), "{texts:?}");

    // The session carries the whole cycle in order.
    let stored = fs::read_to_string(dir.path().join("s.jsonl")).expect("read");
    let events = replay(&stored).expect("replay");
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| match &e.kind {
            vox_session::EventKind::Message { .. } => "message",
            vox_session::EventKind::ToolCall { .. } => "tool_call",
            vox_session::EventKind::ToolResult { .. } => "tool_result",
            _ => "other",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["message", "message", "tool_call", "tool_result", "message"]
    );
}

#[tokio::test]
async fn turn_limit_stops_a_runaway_agent() {
    let dir = project();
    let mut agent = agent(&dir, vec![tool_call_turn(&[read_call()]); 10]);
    let reason = agent.run("loop forever", &mut |_| {}).await.expect("run");
    assert_eq!(reason, FinishReason::TurnLimit);
}

#[tokio::test]
async fn consecutive_tool_errors_stop_the_run() {
    let dir = project();
    let failing = vox_domain::tool::ToolCall {
        id: "c1".to_string(),
        name: "read".to_string(),
        arguments: serde_json::json!({"path": "missing.rs"}),
    };
    let mut agent = agent(&dir, vec![tool_call_turn(&[failing]); 10]);
    let reason = agent
        .run("fail repeatedly", &mut |_| {})
        .await
        .expect("run");
    assert_eq!(reason, FinishReason::ToolErrorLimit);
}
