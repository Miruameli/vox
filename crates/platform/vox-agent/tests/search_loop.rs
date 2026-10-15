//! The search tools through the agent loop: a grep or glob call the
//! model makes comes back as a tool result it can read next turn.

mod fixtures;

use vox_agent::FinishReason;

use fixtures::{agent, glob_call, grep_call, project, stop_turn, tool_call_turn};

#[tokio::test]
async fn a_grep_call_returns_matching_lines_to_the_model() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[grep_call()]), stop_turn("found it")],
    );

    let reason = agent.run("find alpha", &mut |_| {}).await.expect("run");

    assert_eq!(reason, FinishReason::ModelStopped);
    let messages = agent.resume_messages().expect("resume");
    let tool_result = messages
        .iter()
        .flat_map(|message| message.content.iter())
        .find_map(|block| match block {
            vox_domain::message::ContentBlock::ToolResult { content, .. } => Some(content),
            _ => None,
        })
        .expect("tool result in conversation");
    assert!(tool_result.contains("main.rs:1:alpha"), "{tool_result}");
    assert!(!tool_result.contains("beta"), "{tool_result}");
}

#[tokio::test]
async fn a_glob_call_returns_project_paths_to_the_model() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[glob_call()]), stop_turn("found it")],
    );

    let reason = agent.run("list sources", &mut |_| {}).await.expect("run");

    assert_eq!(reason, FinishReason::ModelStopped);
    let messages = agent.resume_messages().expect("resume");
    let tool_result = messages
        .iter()
        .flat_map(|message| message.content.iter())
        .find_map(|block| match block {
            vox_domain::message::ContentBlock::ToolResult { content, .. } => Some(content),
            _ => None,
        })
        .expect("tool result in conversation");
    assert!(tool_result.contains("main.rs"), "{tool_result}");
}
