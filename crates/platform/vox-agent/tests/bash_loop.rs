//! The bash tool through the agent loop: a command the model runs
//! comes back as a tool result it can read next turn.

mod fixtures;

use vox_agent::FinishReason;

use fixtures::{agent, bash_call, project, stop_turn, tool_call_turn};

#[tokio::test]
async fn a_bash_call_returns_the_command_output_to_the_model() {
    let dir = project();
    let mut agent = agent(
        &dir,
        vec![tool_call_turn(&[bash_call()]), stop_turn("ran it")],
    );

    let reason = agent.run("echo alpha", &mut |_| {}).await.expect("run");

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
    assert!(tool_result.contains("exit 0"), "{tool_result}");
    assert!(tool_result.contains("alpha"), "{tool_result}");
}
