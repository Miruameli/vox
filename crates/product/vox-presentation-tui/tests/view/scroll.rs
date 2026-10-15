//! Scrolling: the transcript moves in pages measured from the tail, so
//! new output never moves what the user is reading.

use vox_agent::LoopEvent;
use vox_presentation_tui::app::{scroll_page, App, AppMsg};
use vox_presentation_tui::Header;

/// An app with a visible body of four lines.
fn app() -> App {
    let mut app = App::new(Header {
        model: "mock-model".to_string(),
        session_id: "tui-1".to_string(),
        project_root: "/repo".into(),
    });
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
fn scrolling_measures_from_the_tail_and_clamps_at_both_ends() {
    let mut app = app();
    for index in 0..8 {
        app.apply(text(&format!("line {index}")));
    }
    // A transcript longer than the body starts at the tail.
    assert_eq!(app.scroll_from_tail(), 0);
    scroll_page(&mut app, true);
    assert_eq!(app.scroll_from_tail(), 3);
    for _ in 0..10 {
        scroll_page(&mut app, true);
    }
    // Overscrolling is clamped, so the view cannot walk off the top.
    assert!(app.scroll_from_tail() <= 40);
    for _ in 0..40 {
        scroll_page(&mut app, false);
    }
    assert_eq!(app.scroll_from_tail(), 0);
}

#[test]
fn new_output_never_moves_a_view_that_was_scrolled_back() {
    let mut app = app();
    for index in 0..8 {
        app.apply(text(&format!("line {index}")));
    }
    scroll_page(&mut app, true);
    let position = app.scroll_from_tail();
    app.apply(text("arrives later"));
    assert_eq!(app.scroll_from_tail(), position);
}
