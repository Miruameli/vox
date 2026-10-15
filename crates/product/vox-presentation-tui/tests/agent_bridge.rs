//! The bridge: a prompt typed into the state reaches the agent, and the
//! events it emits come back as conversation entries. No terminal is
//! involved, which is the point - the same path the TUI drives is driven
//! here by hand.

use std::fs;

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;
use vox_agent::{FinishReason, LoopEvent};
use vox_presentation_tui::app::{handle_key, App, AppMsg};
use vox_presentation_tui::{EntryKind, Header, Worker};

mod mock_provider;
use mock_provider::{agent, project, text_turn, tool_turn};

/// A header for these tests.
fn header() -> Header {
    Header {
        model: "mock-model".to_string(),
        session_id: "tui-1".to_string(),
        project_root: "/repo".into(),
    }
}

/// Type `prompt` into the view and submit it, as the prompt line does.
fn submit(app: &mut App, prompt: &str) -> String {
    for character in prompt.chars() {
        handle_key(app, KeyEvent::from(KeyCode::Char(character)));
    }
    match handle_key(app, KeyEvent::from(KeyCode::Enter)) {
        vox_presentation_tui::Action::Submit(prompt) => prompt,
        other => panic!("expected a submit, got {other:?}"),
    }
}

/// Apply messages until the run finishes, and return why.
async fn drain(app: &mut App, inbox: &mut mpsc::UnboundedReceiver<AppMsg>) -> FinishReason {
    while let Some(message) = inbox.recv().await {
        let reason = match &message {
            AppMsg::Agent(LoopEvent::Finished { reason }) => Some(*reason),
            _ => None,
        };
        app.apply(message);
        if let Some(reason) = reason {
            return reason;
        }
    }
    panic!("the worker stopped without finishing the run");
}

#[tokio::test]
async fn a_prompt_shows_the_prompt_the_tool_activity_and_the_answer() {
    let dir = project();
    let (agent, seen) = agent(&dir, vec![tool_turn(), text_turn("all done")]);
    let cancel = agent.cancel_token();
    let (events, mut inbox) = mpsc::unbounded_channel();
    let worker = Worker::spawn(agent, events, cancel, Vec::new());

    let mut app = App::new(header());
    worker
        .submit(submit(&mut app, "read main.rs"))
        .expect("submit");
    assert_eq!(
        drain(&mut app, &mut inbox).await,
        FinishReason::ModelStopped
    );

    let entries = app.entries();
    assert!(entries.iter().any(|entry| entry.text == "read main.rs"));
    assert!(entries
        .iter()
        .any(|entry| entry.kind == EntryKind::Tool && entry.text == "✓ read"));
    assert!(entries.iter().any(|entry| entry.text == "all done"));
    // The tool result preview reaches the view, capped by the loop and
    // never by the view itself.
    assert!(entries.iter().any(|entry| {
        entry
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("alpha"))
    }));
    // The session recorded the whole run, and the provider was asked
    // twice: once for the prompt, once more with the tool result in.
    let stored = fs::read_to_string(dir.path().join("session.jsonl")).expect("session");
    assert!(stored.contains("read main.rs"));
    assert_eq!(
        *seen.lock().expect("lock"),
        vec![1, 3],
        "one message for the prompt, three once the tool result is in"
    );
}

#[tokio::test]
async fn a_second_prompt_keeps_the_first_turn_in_the_conversation() {
    let dir = project();
    let (agent, seen) = agent(&dir, vec![text_turn("first"), text_turn("second")]);
    let cancel = agent.cancel_token();
    let (events, mut inbox) = mpsc::unbounded_channel();
    let worker = Worker::spawn(agent, events, cancel, Vec::new());

    let mut app = App::new(header());
    worker.submit(submit(&mut app, "one")).expect("submit");
    assert_eq!(
        drain(&mut app, &mut inbox).await,
        FinishReason::ModelStopped
    );

    worker.submit(submit(&mut app, "two")).expect("submit");
    assert_eq!(
        drain(&mut app, &mut inbox).await,
        FinishReason::ModelStopped
    );

    // The second request carries the first exchange, which is what
    // makes the surface a conversation instead of two one-shots.
    let requests = seen.lock().expect("lock");
    assert_eq!(requests.len(), 2, "{requests:?}");
    assert!(requests[1] > requests[0], "{requests:?}");
    assert!(app.entries().iter().any(|entry| entry.text == "first"));
    assert!(app.entries().iter().any(|entry| entry.text == "second"));
}

#[tokio::test]
async fn cancelling_stops_the_run_at_the_next_safe_point() {
    let dir = project();
    let (agent, _seen) = agent(&dir, vec![tool_turn(), text_turn("unreachable")]);
    let cancel = agent.cancel_token();
    let (events, mut inbox) = mpsc::unbounded_channel();
    let worker = Worker::spawn(agent, events, cancel, Vec::new());

    let mut app = App::new(header());
    worker
        .submit(submit(&mut app, "read main.rs"))
        .expect("submit");
    worker.cancel();

    assert_eq!(drain(&mut app, &mut inbox).await, FinishReason::Cancelled);
    assert!(app
        .entries()
        .iter()
        .any(|entry| entry.text == "run stopped: cancelled"));
    assert!(app.run_state() == &vox_presentation_tui::app::RunState::Idle);
}
