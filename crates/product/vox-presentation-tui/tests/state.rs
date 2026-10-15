//! The state machine: agent events, scrolling, and what the surface
//! keeps between turns.

use crossterm::event::{KeyCode, KeyEvent};
use vox_agent::{FinishReason, LoopEvent};
use vox_presentation_tui::app::{App, AppMsg, RunState};
use vox_presentation_tui::{EntryKind, Header};
use vox_shared::VoxError;

/// An app for event and scroll tests.
fn app() -> App {
    let mut app = App::new(Header {
        model: "mock-model".to_string(),
        session_id: "tui-1".to_string(),
        project_root: "/repo".into(),
    });
    // The renderer sets this from the terminal; scroll tests need it.
    app.set_body_height(4);
    app
}

/// Text the model produced.
fn text(message: &str) -> AppMsg {
    AppMsg::Agent(LoopEvent::Text {
        text: message.to_string(),
    })
}

#[test]
fn a_turn_tracks_its_number_and_the_running_tool() {
    let mut app = app();
    app.apply(AppMsg::Agent(LoopEvent::TurnStarted { turn: 2 }));
    assert_eq!(
        app.run_state(),
        &RunState::Running {
            turn: 2,
            tool: None
        }
    );
    app.apply(AppMsg::Agent(LoopEvent::ToolStarted {
        name: "read".to_string(),
        detail: "path: main.rs".to_string(),
    }));
    assert_eq!(
        app.run_state(),
        &RunState::Running {
            turn: 2,
            tool: Some("read".to_string())
        }
    );
}

#[test]
fn assistant_text_becomes_an_entry_and_an_empty_turn_is_dropped() {
    let mut app = app();
    app.apply(text("thinking out loud"));
    app.apply(text("   "));
    let assistant: Vec<&str> = app
        .entries()
        .iter()
        .filter(|entry| entry.kind == EntryKind::Assistant)
        .map(|entry| entry.text.as_str())
        .collect();
    assert_eq!(assistant, vec!["thinking out loud"]);
}

#[test]
fn a_finished_tool_records_its_outcome_and_its_detail() {
    let mut app = app();
    app.apply(AppMsg::Agent(LoopEvent::ToolFinished {
        name: "read".to_string(),
        is_error: false,
        detail: "1|a1b2|alpha".to_string(),
    }));
    app.apply(AppMsg::Agent(LoopEvent::ToolFinished {
        name: "write".to_string(),
        is_error: true,
        detail: "cannot write the file".to_string(),
    }));
    let tools: Vec<(&str, bool)> = app
        .entries()
        .iter()
        .filter(|entry| entry.kind == EntryKind::Tool)
        .map(|entry| (entry.text.as_str(), entry.text.starts_with('✗')))
        .collect();
    assert_eq!(tools, vec![("✓ read", false), ("✗ write", true)]);
    assert!(app
        .entries()
        .iter()
        .any(|entry| entry.detail.as_deref() == Some("1|a1b2|alpha")));
}

#[test]
fn hidden_tool_activity_drops_the_entry_outright() {
    use vox_presentation_tui::app::handle_key;
    let mut app = app();
    for character in "/tools".chars() {
        handle_key(&mut app, KeyEvent::from(KeyCode::Char(character)));
    }
    handle_key(&mut app, KeyEvent::from(KeyCode::Enter));
    assert!(!app.shows_tools());

    app.apply(AppMsg::Agent(LoopEvent::ToolFinished {
        name: "read".to_string(),
        is_error: false,
        detail: "alpha".to_string(),
    }));
    assert!(!app
        .entries()
        .iter()
        .any(|entry| entry.kind == EntryKind::Tool));

    handle_key(&mut app, KeyEvent::from(KeyCode::Char('x')));
    assert!(app.input().text() == "x");
}

#[test]
fn a_clean_stop_is_silent_and_any_other_stop_announces_itself() {
    let mut app = app();
    app.apply(AppMsg::Agent(LoopEvent::Finished {
        reason: FinishReason::ModelStopped,
    }));
    assert!(!app
        .entries()
        .iter()
        .any(|entry| entry.text.starts_with("run stopped")));
    assert_eq!(app.run_state(), &RunState::Idle);

    app.apply(AppMsg::Agent(LoopEvent::Finished {
        reason: FinishReason::TurnLimit,
    }));
    assert!(app
        .entries()
        .iter()
        .any(|entry| entry.text == "run stopped: the turn limit was reached"));
}

#[test]
fn an_infrastructure_failure_shows_the_generic_message_and_keeps_the_state_usable() {
    let mut app = app();
    app.apply(AppMsg::Failed(VoxError::Provider {
        detail: "POST /v1/chat/completions: connection refused".to_string(),
    }));
    assert_eq!(app.run_state(), &RunState::Idle);
    assert!(app
        .entries()
        .iter()
        .any(|entry| entry.text == "provider request failed"));
    // The diagnostic detail never reaches the view.
    assert!(!app
        .entries()
        .iter()
        .any(|entry| entry.text.contains("connection refused")));
}

#[test]
fn a_tick_advances_the_running_indicator() {
    let mut app = app();
    let first = app.spinner();
    app.apply(AppMsg::Tick);
    assert_eq!(app.spinner(), first + 1);
}
