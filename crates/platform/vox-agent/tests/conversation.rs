//! Continuing a conversation across prompts.
//!
//! `Agent::run` extends the agent's own conversation, so a second
//! `run` in one process sees everything the first one produced;
//! `continue_conversation` is what a surface that lives for many
//! prompts and owns its history uses, so the earlier turns stay in
//! the next request and a failed turn still leaves its history with
//! the caller.

mod fixtures;

/// Two turns that end the run with text.
fn two_turns() -> Vec<vox_domain::completion::AssistantTurn> {
    vec![fixtures::stop_turn("first"), fixtures::stop_turn("second")]
}

#[tokio::test]
async fn a_second_run_continues_the_agents_conversation() {
    let dir = fixtures::project();
    let (mut agent, requests) = fixtures::agent_recording(&dir, two_turns());

    agent.run("first prompt", &mut |_| {}).await.expect("first");
    agent
        .run("second prompt", &mut |_| {})
        .await
        .expect("second");

    // Both exchanges reached the provider inside one agent: the
    // second request carries the first one's turns. Regression:
    // an earlier loop rebuilt its message list locally, so every
    // prompt after the first was sent with no history at all.
    let requests = requests.lock().expect("lock");
    assert_eq!(requests.len(), 2, "{requests:?}");
    let second = &requests[1];
    let texts: Vec<String> = second
        .messages
        .iter()
        .map(|message| message.text_content())
        .collect();
    assert!(
        texts.iter().any(|text| text.contains("first prompt")),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("second prompt")),
        "{texts:?}"
    );
    assert!(texts.iter().any(|text| text.contains("first")), "{texts:?}");
}

#[tokio::test]
async fn continuing_keeps_the_earlier_turn_with_the_caller() {
    let dir = fixtures::project();
    let mut agent = fixtures::agent(&dir, two_turns());
    let mut history: Vec<vox_domain::message::Message> = Vec::new();

    agent
        .continue_conversation(&mut history, "one", &mut |_| {})
        .await
        .expect("first");
    assert_eq!(history.len(), 2);

    agent
        .continue_conversation(&mut history, "two", &mut |_| {})
        .await
        .expect("second");
    assert_eq!(history.len(), 4);
    // The second exchange starts from the first one, which is what
    // makes the surface a conversation instead of two one-shots.
    let stored = std::fs::read_to_string(dir.path().join("s.jsonl")).expect("session");
    assert!(stored.contains("one"));
    assert!(stored.contains("first"));
    assert!(stored.contains("two"));
    assert!(stored.contains("second"));
}

#[tokio::test]
async fn a_tool_round_trip_keeps_its_result_in_the_history() {
    let dir = fixtures::project();
    let mut agent = fixtures::agent(
        &dir,
        vec![
            fixtures::tool_call_turn(&[fixtures::read_call()]),
            fixtures::stop_turn("done"),
            fixtures::stop_turn("again"),
        ],
    );
    let mut history: Vec<vox_domain::message::Message> = Vec::new();

    agent
        .continue_conversation(&mut history, "read the file", &mut |_| {})
        .await
        .expect("first");
    // The exchange is the prompt, the assistant turn that called the
    // tool, the result that answers it, and the closing turn. Without
    // the result the next request would carry an unanswered tool call,
    // which providers reject.
    assert_eq!(history.len(), 4);

    agent
        .continue_conversation(&mut history, "and again", &mut |_| {})
        .await
        .expect("second");
    assert_eq!(history.len(), 6);
}

#[tokio::test]
async fn a_failed_turn_still_leaves_its_history_with_the_caller() {
    let dir = fixtures::project();
    let mut agent = fixtures::agent(&dir, vec![fixtures::stop_turn("only")]);
    let mut history: Vec<vox_domain::message::Message> = Vec::new();

    agent
        .continue_conversation(&mut history, "one", &mut |_| {})
        .await
        .expect("first");
    let reason = agent
        .continue_conversation(&mut history, "two", &mut |_| {})
        .await
        .expect_err("the mock provider ran out of turns");
    assert_eq!(reason.to_string(), "stream decode failed");
    // The failed turn's request and its own user message are both
    // still there, so the caller can retry without replaying the file.
    assert_eq!(history.len(), 3);
}
