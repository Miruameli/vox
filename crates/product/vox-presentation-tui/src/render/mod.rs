//! One frame of the surface.
//!
//! The frame is five stacked rows: header, transcript, status, prompt,
//! and key hints. The transcript takes everything the other four leave
//! over, so a resize only ever trades transcript height for nothing
//! else, and no panel can be pushed off screen by another.

pub mod panels;
pub mod styles;
pub mod wrap;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::App;

/// Draw one frame of the surface.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());
    panels::header(frame, rows[0], app);
    panels::transcript(frame, rows[1], app);
    panels::status(frame, rows[2], app);
    panels::input(frame, rows[3], app);
    panels::hints(frame, rows[4], app);
}
