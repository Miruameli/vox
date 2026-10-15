//! Slash commands typed into the prompt line.
//!
//! A line starting with `/` is never sent to the model: it is resolved
//! here and answered by the view ([`execute`]). Anything else is a
//! prompt for the model. Commands are matched case-insensitively and
//! tolerate surrounding spaces, so `/Help` and ` /help ` both work.

use super::keys::Action;
use super::{App, Entry, EntryKind, RunState};

/// A command resolved from the prompt line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Leave the surface.
    Quit,
    /// Show the key and command reference.
    Help,
    /// Empty the transcript and the recalled prompts.
    Clear,
    /// Stop the running turn.
    Cancel,
    /// Show or hide tool activity.
    Tools,
    /// Expand or collapse the detail of tool entries.
    Expand,
}

/// What one submitted line means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parsed<'a> {
    /// Text for the model.
    Prompt(&'a str),
    /// A resolved command.
    Command(Command),
    /// A `/` line that names no known command.
    Unknown(&'a str),
}

/// Resolve one submitted line.
pub fn parse(line: &str) -> Parsed<'_> {
    let trimmed = line.trim();
    let Some(rest) = trimmed.strip_prefix('/') else {
        return Parsed::Prompt(trimmed);
    };
    match rest.trim().to_ascii_lowercase().as_str() {
        "quit" | "q" | "exit" => Parsed::Command(Command::Quit),
        "help" | "h" | "?" => Parsed::Command(Command::Help),
        "clear" => Parsed::Command(Command::Clear),
        "cancel" => Parsed::Command(Command::Cancel),
        "tools" => Parsed::Command(Command::Tools),
        "expand" => Parsed::Command(Command::Expand),
        _ => Parsed::Unknown(trimmed),
    }
}

/// Carry out one resolved line.
///
/// A prompt becomes a user entry plus a submit action; a command
/// answers itself in the transcript. The view never sends a command
/// to the model.
pub(crate) fn execute(app: &mut App, parsed: Parsed<'_>) -> Action {
    match parsed {
        Parsed::Prompt(prompt) => {
            if prompt.is_empty() {
                return Action::Nothing;
            }
            app.recall.push(prompt);
            app.transcript.push(Entry::new(EntryKind::User, prompt));
            Action::Submit(prompt.to_string())
        }
        Parsed::Command(Command::Quit) => Action::Quit,
        Parsed::Command(Command::Help) => {
            app.notice(help_text());
            Action::Redraw
        }
        Parsed::Command(Command::Clear) => {
            app.transcript.clear();
            app.recall.clear();
            Action::Redraw
        }
        Parsed::Command(Command::Cancel) => {
            if matches!(app.state, RunState::Running { .. }) {
                app.cancelling = true;
                return Action::Cancel;
            }
            app.notice("no run to cancel");
            Action::Nothing
        }
        Parsed::Command(Command::Tools) => {
            app.show_tools = !app.show_tools;
            app.notice(format!(
                "tool activity {}",
                if app.show_tools { "shown" } else { "hidden" }
            ));
            Action::Redraw
        }
        Parsed::Command(Command::Expand) => {
            app.expand_tools = !app.expand_tools;
            app.notice(format!(
                "tool detail {}",
                if app.expand_tools {
                    "expanded"
                } else {
                    "collapsed"
                }
            ));
            Action::Redraw
        }
        Parsed::Unknown(name) => {
            app.notice(format!("unknown command {name}; /help lists the commands"));
            Action::Nothing
        }
    }
}

/// The reference shown by `/help`.
fn help_text() -> String {
    [
        "Enter  send the prompt           Up/Down  recall earlier prompts",
        "Esc    clear the prompt line     PgUp/PgDn  scroll the transcript",
        "Ctrl+O expand tool detail        Ctrl+U  clear the line",
        "Ctrl+C cancel a run, clear a draft, or quit",
        "/quit  leave    /cancel  stop the run    /clear  clear the view",
        "/tools show or hide tool activity        /expand tool detail",
        "/help  show this reference",
    ]
    .join("\n")
}
