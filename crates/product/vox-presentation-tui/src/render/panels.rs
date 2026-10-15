//! The five panels of one frame.
//!
//! Each panel receives a rectangle the layout already computed and
//! writes into it. The transcript panel is the only one that writes
//! back into the state: it records the visible body height, which is
//! what page scrolling and the scroll indicator need. Panels never
//! decide anything - everything they show was decided by the keys and
//! the agent events.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::app::events::reason_text;
use crate::app::{App, RunState};
use crate::render::styles;
use crate::render::wrap;

/// Header: product, model, session, and working directory.
pub fn header(frame: &mut Frame, area: Rect, app: &App) {
    let spans = vec![
        Span::styled(" vox ", styles::chrome()),
        Span::raw(app.header.model.clone()),
        Span::styled("  ·  ", styles::chrome()),
        Span::raw(app.header.session_id.clone()),
        Span::styled("  ·  ", styles::chrome()),
        Span::raw(app.header.project_root.display().to_string()),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Transcript: the conversation, scrolled to the tail by default.
pub fn transcript(frame: &mut Frame, area: Rect, app: &mut App) {
    app.body_height = area.height as usize;
    let lines = wrap::display_lines(&app.transcript, area.width as usize, app.expand_tools);
    let max_offset = lines.len().saturating_sub(app.body_height);
    // Scrolling is measured from the tail, so new output never moves
    // what the user is reading unless they are already at the tail.
    let offset = max_offset.saturating_sub(app.scroll_from_tail);
    let rendered: Vec<Line> = lines
        .into_iter()
        .map(|entry| {
            if entry.first {
                Line::from(vec![
                    Span::styled(entry.kind.label(), styles::label(entry.kind)),
                    Span::raw(" "),
                    Span::styled(entry.text, styles::body(entry.kind)),
                ])
            } else {
                Line::from(Span::styled(entry.text, styles::body(entry.kind)))
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(rendered).scroll((offset as u16, 0)), area);
}

/// Status bar: what the run is doing, and how far the view is scrolled.
pub fn status(frame: &mut Frame, area: Rect, app: &App) {
    let left = match &app.state {
        RunState::Idle => match app.last_reason {
            Some(reason) => format!("idle · {}", reason_text(reason)),
            None => "idle".to_string(),
        },
        RunState::Running { turn, tool } => {
            let frame = spin(app.spinner);
            match tool {
                Some(name) => format!("{frame} turn {turn} · {name}"),
                None => format!("{frame} turn {turn} · thinking"),
            }
        }
    };
    let left = if app.cancelling {
        format!("{left} · stopping")
    } else {
        left
    };
    let right = if app.scroll_from_tail == 0 {
        String::new()
    } else {
        format!("scrolled up {} ", app.scroll_from_tail)
    };
    let width = area.width as usize;
    let gap = width.saturating_sub(left.width() + right.width());
    let bar = Line::from(vec![
        Span::raw(left),
        Span::raw(" ".repeat(gap)),
        Span::raw(right),
    ]);
    frame.render_widget(Paragraph::new(bar).style(styles::status()), area);
}

/// Prompt area: the line being edited, with the hardware cursor on it.
pub fn input(frame: &mut Frame, area: Rect, app: &App) {
    let focused = !matches!(app.state, RunState::Running { .. });
    let block = Block::bordered()
        .title(" prompt ")
        .border_style(styles::prompt_border(focused));
    let inner = block.inner(area);
    let (text, cursor) = app.input.view(inner.width as usize);
    let shown = if text.is_empty() {
        Span::styled(
            "type a prompt, /help for keys and commands",
            styles::placeholder(),
        )
    } else {
        Span::raw(text)
    };
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(Line::from(shown)), inner);
    frame.set_cursor_position((inner.x + cursor as u16, inner.y));
}

/// Footer: the keys worth remembering, dimmed out of the way.
pub fn hints(frame: &mut Frame, area: Rect, app: &App) {
    let text = "Enter send · PgUp/PgDn scroll · /help commands · Ctrl+C cancel or quit · Ctrl+O tool detail";
    let text = if app.expand_tools {
        format!("{text} · detail expanded")
    } else {
        text.to_string()
    };
    frame.render_widget(Paragraph::new(Span::styled(text, styles::chrome())), area);
}

/// Frames of the running indicator.
const SPINNER: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠇"];

/// The indicator frame for `tick`.
fn spin(tick: usize) -> &'static str {
    SPINNER[tick % SPINNER.len()]
}
