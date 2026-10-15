//! Key handling: what one key press does to the view.
//!
//! [`handle_key`] maps a key to an [`Action`]: either a transition the
//! state machine applies on its own, or something only the event loop
//! can do - send a prompt to the worker, cancel a run, redraw, or
//! leave. Commands are read in the prompt line only, so a command can
//! never be mistaken for input to the model.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, RunState};
use crate::app::commands;

/// What the event loop owes after one key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Nothing outside the view changed.
    Nothing,
    /// Send this prompt to the agent worker.
    Submit(String),
    /// Stop the running turn.
    Cancel,
    /// Redraw the frame, for example after a terminal resize.
    Redraw,
    /// Restore the terminal and exit.
    Quit,
}

/// Apply one key press to the view.
pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    use KeyCode::{
        Backspace, Char, Delete, Down, End, Enter, Esc, Home, Left, PageDown, PageUp, Right, Up,
    };
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            Char('c') => return interrupt(app),
            Char('d') => return Action::Quit,
            Char('o') => {
                app.expand_tools = !app.expand_tools;
                return Action::Redraw;
            }
            Char('u') => return effect(app, |app| app.input.clear()),
            Char('l') => return Action::Redraw,
            _ => {}
        }
    }
    match key.code {
        Enter => submit(app),
        Esc => effect(app, |app| app.input.clear()),
        Backspace => effect(app, |app| app.input.backspace()),
        Delete => effect(app, |app| app.input.delete()),
        Left => effect(app, |app| app.input.move_left()),
        Right => effect(app, |app| app.input.move_right()),
        Home => effect(app, |app| app.input.move_home()),
        End => effect(app, |app| app.input.move_end()),
        Up => effect(app, step_back),
        Down => effect(app, step_forward),
        PageUp => effect(app, |app| scroll_page(app, true)),
        PageDown => effect(app, |app| scroll_page(app, false)),
        Char(character) => effect(app, |app| app.input.insert(character)),
        _ => Action::Nothing,
    }
}

/// Run one transition of the view and report no side effect.
fn effect(app: &mut App, transition: impl FnOnce(&mut App)) -> Action {
    transition(app);
    Action::Nothing
}

/// The interrupt key: cancel a run, clear a draft, or leave.
///
/// A running turn is stopped first, so an accidental exit never
/// happens while the agent is still working. Otherwise an unsent line
/// is cleared - recoverable with Up - and an empty prompt line leaves.
fn interrupt(app: &mut App) -> Action {
    if matches!(app.state, RunState::Running { .. }) {
        app.cancelling = true;
        return Action::Cancel;
    }
    if !app.input.is_empty() {
        app.input.clear();
        return Action::Nothing;
    }
    Action::Quit
}

/// Resolve one submitted line: a prompt, a command, or a mistake.
fn submit(app: &mut App) -> Action {
    let line = app.input.take();
    commands::execute(app, commands::parse(&line))
}

/// Step one recalled prompt back into the prompt line.
fn step_back(app: &mut App) {
    let draft = app.input.text();
    if let Some(prompt) = app.recall.previous(&draft) {
        app.input.set_text(&prompt);
    }
}

/// Step one recalled prompt forward into the prompt line.
fn step_forward(app: &mut App) {
    if let Some(prompt) = app.recall.forward() {
        app.input.set_text(&prompt);
    }
}

/// Scroll the transcript by one page, backwards or towards the tail.
pub fn scroll_page(app: &mut App, backwards: bool) {
    let step = app.body_height.saturating_sub(1).max(1);
    app.scroll_from_tail = if backwards {
        app.scroll_from_tail.saturating_add(step)
    } else {
        app.scroll_from_tail.saturating_sub(step)
    };
}
