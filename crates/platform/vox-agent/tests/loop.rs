//! Loop behaviour: one full tool cycle, the stop path, and the turn and
//! tool-error bounds, with a scripted mock provider.

mod fixtures;

use std::fs;

use vox_agent::{FinishReason, LoopEvent};
use vox_domain::message::{ContentBlock, Message, Role};
use vox_session::replay;

use fixtures::{agent, agent_recording, project, read_call, stop_turn, tool_call_turn};

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
    assert!(events.iter().any(|e| matches!(
        e,
        LoopEvent::ToolStarted { name, .. } if name == "read"
    )));
    // Tool events carry a bounded preview of what the tool saw, not
    // the whole output: the session file holds the full result.
    assert!(events.iter().any(|e| matches!(
        e,
        LoopEvent::ToolFinished { name, is_error, detail }
            if name == "read" && !is_error && !detail.trim().is_empty()
    )));
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

#[tokio::test]
async fn a_resumed_run_sends_prior_messages_before_the_prompt() {
    let dir = project();
    let (agent, requests) = agent_recording(&dir, vec![stop_turn("done")]);
    let prior = vec![
        Message::user("earlier question"),
        Message {
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: "earlier answer".to_string(),
            }],
        },
    ];
    let mut agent = agent.with_prior(prior.clone());
    let reason = agent.run("follow up", &mut |_| {}).await.expect("run");
    assert_eq!(reason, FinishReason::ModelStopped);

    let messages = requests
        .lock()
        .expect("lock")
        .last()
        .cloned()
        .expect("no request recorded")
        .messages;
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0], prior[0]);
    assert_eq!(messages[1], prior[1]);
    assert_eq!(messages[2].text_content(), "follow up");
}

#[tokio::test]
async fn a_second_run_continues_the_same_conversation() {
    // An interactive session calls run() per input line; the second
    // prompt must reach the provider with the whole conversation.
    let dir = project();
    let (mut agent, requests) = agent_recording(
        &dir,
        vec![stop_turn("first answer"), stop_turn("second answer")],
    );
    agent.run("prompt one", &mut |_| {}).await.expect("first");
    agent.run("prompt two", &mut |_| {}).await.expect("second");

    let log = requests.lock().expect("lock");
    assert_eq!(log.len(), 2);
    let second = log.last().expect("second request");
    let roles: Vec<Role> = second.messages.iter().map(|m| m.role).collect();
    assert_eq!(roles, vec![Role::User, Role::Assistant, Role::User]);
    assert_eq!(second.messages[0].text_content(), "prompt one");
    assert!(second.messages[1].text_content().contains("first answer"));
    assert_eq!(second.messages[2].text_content(), "prompt two");
}
