//! Rendering: one frame drawn into a test backend, so the layout is
//! verified without a terminal.

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;
use vox_agent::LoopEvent;
use vox_presentation_tui::app::{App, AppMsg};
use vox_presentation_tui::render::draw;
use vox_presentation_tui::{EntryKind, Header};

/// An app with two entries and a prompt line, at a known size.
fn framed() -> (App, Terminal<TestBackend>) {
    let mut app = App::new(Header {
        model: "mock-model".to_string(),
        session_id: "tui-1".to_string(),
        project_root: "/repo".into(),
    });
    app.apply(AppMsg::Agent(LoopEvent::Text {
        text: "first answer".to_string(),
    }));
    app.apply(AppMsg::Agent(LoopEvent::ToolFinished {
        name: "read".to_string(),
        is_error: false,
        detail: "path: main.rs".to_string(),
    }));
    app.apply(AppMsg::Agent(LoopEvent::Finished {
        reason: vox_agent::FinishReason::ModelStopped,
    }));
    let mut terminal = Terminal::new(TestBackend::new(80, 12)).expect("terminal");
    // Draw once so the renderer records the visible body height.
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    (app, terminal)
}

/// The text of one drawn row.
fn row(buffer: &Buffer, row: u16) -> String {
    (0..buffer.area.width)
        .map(|column| buffer[(column, row)].symbol().to_string())
        .collect::<Vec<_>>()
        .join("")
        .trim_end()
        .to_string()
}

/// Every row of the frame, trimmed.
fn frame(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer().clone();
    (0..buffer.area.height)
        .map(|row_index| row(&buffer, row_index))
        .collect()
}

#[test]
fn a_frame_shows_the_header_the_transcript_and_the_status() {
    let (mut app, mut terminal) = framed();
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    let rows = frame(&terminal);

    assert!(rows[0].contains("mock-model"), "{rows:?}");
    assert!(rows[0].contains("tui-1"));
    assert!(rows[0].contains("/repo"));
    assert!(rows.iter().any(|row| row.contains("vox first answer")));
    assert!(rows.iter().any(|row| row.contains("tool ✓ read")));
    assert!(rows
        .iter()
        .any(|row| row.contains("idle · the model stopped")));
    assert!(app.body_height() > 0);
}

#[test]
fn the_prompt_area_shows_what_is_typed_and_a_placeholder_when_empty() {
    let (mut app, mut terminal) = framed();
    assert!(frame(&terminal).iter().any(|row| row.contains("prompt")));

    app.input_mut().set_text("read the parser");
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    let rows = frame(&terminal);
    assert!(rows.iter().any(|row| row.contains("read the parser")));
    assert!(!rows
        .iter()
        .any(|row| row.contains("type a prompt, /help for keys")));

    app.input_mut().clear();
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    assert!(frame(&terminal)
        .iter()
        .any(|row| row.contains("type a prompt, /help for keys")));
}

#[test]
fn a_running_turn_shows_the_indicator_and_the_tool_name() {
    let (mut app, mut terminal) = framed();
    app.apply(AppMsg::Agent(LoopEvent::TurnStarted { turn: 3 }));
    app.apply(AppMsg::Agent(LoopEvent::ToolStarted {
        name: "grep".to_string(),
        detail: "pattern: fn main".to_string(),
    }));
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    let rows = frame(&terminal);
    assert!(rows
        .iter()
        .any(|row| row.contains("turn 3") && row.contains("grep")));
}

#[test]
fn a_scrolled_view_says_so_in_the_status_bar() {
    let (mut app, mut terminal) = framed();
    for index in 0..30 {
        app.apply(AppMsg::Agent(LoopEvent::Text {
            text: format!("filler {index}"),
        }));
    }
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    vox_presentation_tui::app::scroll_page(&mut app, true);
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    let rows = frame(&terminal);
    assert!(
        rows.iter().any(|row| row.contains("scrolled up")),
        "{rows:?}"
    );
}

#[test]
fn expanded_tool_detail_appears_under_the_tool_line() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use vox_presentation_tui::app::handle_key;

    let (mut app, mut terminal) = framed();
    let entry = app
        .entries()
        .iter()
        .find(|entry| entry.kind == EntryKind::Tool)
        .expect("tool entry");
    assert_eq!(entry.detail.as_deref(), Some("path: main.rs"));

    // Ctrl+O expands tool detail, exactly as the key binding does.
    let expand = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key(&mut app, expand),
        vox_presentation_tui::app::Action::Redraw
    );
    terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
    assert!(frame(&terminal)
        .iter()
        .any(|row| row.contains("path: main.rs")));
}
