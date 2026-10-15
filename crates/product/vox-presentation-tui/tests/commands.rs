//! Slash commands: a `/` line never reaches the model.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use vox_presentation_tui::app::commands::{parse, Command, Parsed};
use vox_presentation_tui::app::{handle_key, Action, App, RunState};
use vox_presentation_tui::Header;

/// An app with the header filled in, for key and command tests.
fn app() -> App {
    App::new(Header {
        model: "mock-model".to_string(),
        session_id: "tui-1".to_string(),
        project_root: "/repo".into(),
    })
}

/// Press one key, as a terminal reports it.
fn press(ctrl: bool, code: KeyCode) -> KeyEvent {
    let mut modifiers = KeyModifiers::NONE;
    if ctrl {
        modifiers |= KeyModifiers::CONTROL;
    }
    KeyEvent::new(code, modifiers)
}

#[test]
fn plain_lines_are_prompts_and_are_trimmed() {
    assert_eq!(
        parse("  refactor the parser  "),
        Parsed::Prompt("refactor the parser")
    );
}

#[test]
fn commands_resolve_case_insensitively_and_ignore_spaces() {
    assert_eq!(parse("/QUIT"), Parsed::Command(Command::Quit));
    assert_eq!(parse(" /help "), Parsed::Command(Command::Help));
    assert_eq!(parse("/tools"), Parsed::Command(Command::Tools));
    assert_eq!(parse("/expand"), Parsed::Command(Command::Expand));
}

#[test]
fn unknown_commands_are_reported_back_verbatim() {
    assert_eq!(parse("/deploy"), Parsed::Unknown("/deploy"));
}

#[test]
fn a_prompt_is_submitted_once_and_shown_as_a_user_entry() {
    let mut app = app();
    for character in "read the file".chars() {
        handle_key(&mut app, press(false, KeyCode::Char(character)));
    }
    let action = handle_key(&mut app, press(false, KeyCode::Enter));
    assert_eq!(action, Action::Submit("read the file".to_string()));
    assert!(app
        .entries()
        .iter()
        .any(|entry| entry.text == "read the file"));
    assert!(app.input().is_empty());
}

#[test]
fn an_empty_line_is_never_submitted() {
    let mut app = app();
    assert_eq!(
        handle_key(&mut app, press(false, KeyCode::Enter)),
        Action::Nothing
    );
}

#[test]
fn a_command_answers_itself_and_never_submits() {
    let mut app = app();
    for character in "/help".chars() {
        handle_key(&mut app, press(false, KeyCode::Char(character)));
    }
    assert_eq!(
        handle_key(&mut app, press(false, KeyCode::Enter)),
        Action::Redraw
    );
    assert!(app
        .entries()
        .iter()
        .any(|entry| entry.text.contains("Ctrl+C")));
    assert!(!app.entries().iter().any(|entry| entry.text == "/help"));
}

#[test]
fn quit_commands_and_the_eof_key_leave() {
    let mut app = app();
    for character in "/quit".chars() {
        handle_key(&mut app, press(false, KeyCode::Char(character)));
    }
    assert_eq!(
        handle_key(&mut app, press(false, KeyCode::Enter)),
        Action::Quit
    );
    assert_eq!(
        handle_key(&mut app, press(true, KeyCode::Char('d'))),
        Action::Quit
    );
}

#[test]
fn tools_and_expand_commands_flip_their_flag_and_announce_it() {
    let mut app = app();
    for character in "/tools".chars() {
        handle_key(&mut app, press(false, KeyCode::Char(character)));
    }
    assert_eq!(
        handle_key(&mut app, press(false, KeyCode::Enter)),
        Action::Redraw
    );
    assert!(!app.shows_tools());

    for character in "/expand".chars() {
        handle_key(&mut app, press(false, KeyCode::Char(character)));
    }
    assert_eq!(
        handle_key(&mut app, press(false, KeyCode::Enter)),
        Action::Redraw
    );
    assert!(app.expands_tools());
}

#[test]
fn the_interrupt_key_cancels_a_run_before_clearing_a_draft() {
    let mut app = app();
    // Idle with a draft: the first interrupt clears the draft.
    app.input_mut().set_text("half a prompt");
    assert_eq!(
        handle_key(&mut app, press(true, KeyCode::Char('c'))),
        Action::Nothing
    );
    assert!(app.input().is_empty());

    // Running: the interrupt cancels the run instead of leaving.
    app.apply(vox_presentation_tui::AppMsg::Agent(
        vox_agent::LoopEvent::TurnStarted { turn: 1 },
    ));
    assert_eq!(
        handle_key(&mut app, press(true, KeyCode::Char('c'))),
        Action::Cancel
    );
    assert!(app.is_cancelling());
    assert!(matches!(app.run_state(), RunState::Running { .. }));
}
