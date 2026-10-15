//! The tools through the agent loop: a tool call the model makes
//! comes back as a tool result it can read on the next turn.

mod fixtures;

use vox_agent::FinishReason;
use vox_domain::message::{ContentBlock, Message};

use fixtures::{agent, bash_call, glob_call, grep_call, project, stop_turn, tool_call_turn};

/// The text of the first tool result in the conversation a run built.
fn first_tool_result(messages: &[Message]) -> String {
    messages
        .iter()
        .flat_map(|message| message.content.iter())
        .find_map(|block| match block {
            ContentBlock::ToolResult { content, .. } => Some(content.clone()),
            _ => None,
        })
        .expect("tool result in conversation")
}

#[tokio::test]
async fn a_grep_call_returns_matching_lines_to_the_model() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[grep_call()]), stop_turn("found it")],
    );

    let reason = agent.run("find alpha", &mut |_| {}).await.expect("run");

    assert_eq!(reason, FinishReason::ModelStopped);
    let tool_result = first_tool_result(&agent.resume_messages().expect("resume"));
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
    let tool_result = first_tool_result(&agent.resume_messages().expect("resume"));
    assert!(tool_result.contains("main.rs"), "{tool_result}");
}

#[tokio::test]
async fn a_bash_call_returns_the_command_output_to_the_model() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[bash_call()]), stop_turn("ran it")],
    );

    let reason = agent.run("echo alpha", &mut |_| {}).await.expect("run");

    assert_eq!(reason, FinishReason::ModelStopped);
    let tool_result = first_tool_result(&agent.resume_messages().expect("resume"));
    assert!(tool_result.contains("exit 0"), "{tool_result}");
    assert!(tool_result.contains("alpha"), "{tool_result}");
}
