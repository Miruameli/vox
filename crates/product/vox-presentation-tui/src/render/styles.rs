//! Semantic styles for the surface.
//!
//! The surface colours nothing: every style is a modifier (bold, dim,
//! reversed) over the terminal's own palette, so the UI follows the
//! reader's theme instead of fighting it, and a light and a dark
//! terminal both stay readable. When a real theme surface is needed it
//! plugs in here and nowhere else.

use ratatui::style::{Modifier, Style};

use crate::app::transcript::EntryKind;

/// Chrome that surrounds the conversation: header and key hints.
pub fn chrome() -> Style {
    Style::new().add_modifier(Modifier::DIM)
}

/// The label that starts an entry: `you`, `vox`, `tool`.
pub fn label(kind: EntryKind) -> Style {
    match kind {
        EntryKind::User | EntryKind::Assistant => Style::new().add_modifier(Modifier::BOLD),
        EntryKind::Tool | EntryKind::System => chrome(),
    }
}

/// Body text of an entry.
pub fn body(kind: EntryKind) -> Style {
    match kind {
        EntryKind::User | EntryKind::Assistant => Style::new(),
        EntryKind::Tool => chrome(),
        EntryKind::System => chrome(),
    }
}

/// The status bar, which inverts the terminal's colours.
pub fn status() -> Style {
    Style::new().add_modifier(Modifier::REVERSED)
}

/// Border of the prompt area: focused while the run is idle.
pub fn prompt_border(focused: bool) -> Style {
    if focused {
        Style::new().add_modifier(Modifier::BOLD)
    } else {
        chrome()
    }
}

/// Placeholder shown while the prompt line is empty.
pub fn placeholder() -> Style {
    chrome()
}
