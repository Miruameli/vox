//! Cancellation between turns and resume reconstructing the
//! conversation from the session file.

mod fixtures;

use std::sync::atomic::Ordering;

use vox_agent::FinishReason;
use vox_domain::message::{ContentBlock, Role};
use vox_domain::tool::ToolDefinition;

use fixtures::{agent, project, read_call, stop_turn, tool_call_turn};

#[tokio::test]
async fn cancellation_stops_between_turns() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[read_call()]), stop_turn("done")],
    );
    agent.cancel_token().store(true, Ordering::SeqCst);
    let reason = agent.run("cancel me", &mut |_| {}).await.expect("run");
    assert_eq!(reason, FinishReason::Cancelled);
}

#[tokio::test]
async fn resume_rebuilds_the_conversation_from_the_session() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[read_call()]), stop_turn("done")],
    );
    agent.run("read the file", &mut |_| {}).await.expect("run");

    let messages = agent.resume_messages().expect("resume");
    let roles: Vec<Role> = messages.iter().map(|m| m.role).collect();
    assert_eq!(
        roles,
        vec![Role::User, Role::Assistant, Role::Tool, Role::Assistant]
    );
    match &messages[1].content[0] {
        ContentBlock::ToolCall { name, .. } => assert_eq!(name, "read"),
        other => panic!("unexpected: {other:?}"),
    }
    match &messages[2].content[0] {
        ContentBlock::ToolResult { content, .. } => {
            assert!(content.contains("alpha"), "tool output lost: {content}");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn tool_definitions_are_exported_for_providers() {
    let definitions: Vec<ToolDefinition> = vox_engine_tools::definitions();
    let names: Vec<&str> = definitions.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, vec!["read", "edit", "write", "grep", "glob"]);
    assert!(definitions.iter().all(|d| d.parameters.is_object()));
}
