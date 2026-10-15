//! Agent events: what one message from the loop does to the view.
//!
//! The loop is the only writer of these transitions, and every field it
//! touches is here rather than in [`super::mod`] so the state the user
//! drives stays in one place. Two payloads never reach the view
//! unbounded: tool output arrives already previewed by the loop, and
//! an infrastructure failure arrives as the generic message the error
//! type carries - its diagnostic detail goes to the log instead.

use vox_agent::{FinishReason, LoopEvent};
use vox_shared::VoxError;

use super::{App, Entry, EntryKind, RunState};

/// One message from the agent bridge to the view.
#[derive(Debug)]
pub enum AppMsg {
    /// A progress event of the running loop.
    Agent(LoopEvent),
    /// One animation frame passed; advances the running indicator.
    Tick,
    /// The run failed with an infrastructure error.
    Failed(VoxError),
}

/// Apply one message from the agent bridge.
pub fn apply(app: &mut App, message: AppMsg) {
    match message {
        AppMsg::Agent(event) => apply_event(app, event),
        AppMsg::Tick => app.spinner = app.spinner.wrapping_add(1),
        AppMsg::Failed(error) => failed(app, error),
    }
}

/// Apply one loop event.
fn apply_event(app: &mut App, event: LoopEvent) {
    match event {
        LoopEvent::TurnStarted { turn } => {
            app.state = RunState::Running { turn, tool: None };
        }
        // A turn that only calls tools emits no text; an empty entry
        // would be noise, so it is dropped here.
        LoopEvent::Text { text } => {
            if !text.trim().is_empty() {
                app.transcript.push(Entry::new(EntryKind::Assistant, text));
            }
        }
        LoopEvent::ToolStarted { name, .. } => {
            if let RunState::Running { tool, .. } = &mut app.state {
                *tool = Some(name);
            }
        }
        LoopEvent::ToolFinished {
            name,
            is_error,
            detail,
        } => {
            // Hidden while the user turned tool activity off, or while
            // detail stays collapsed: the session file keeps both.
            if !app.show_tools {
                return;
            }
            let mark = if is_error { "✗" } else { "✓" };
            app.transcript.push(Entry::with_detail(
                EntryKind::Tool,
                format!("{mark} {name}"),
                detail,
            ));
        }
        LoopEvent::Finished { reason } => finished(app, reason),
    }
}

/// Record the end of a run.
fn finished(app: &mut App, reason: FinishReason) {
    app.state = RunState::Idle;
    app.cancelling = false;
    app.last_reason = Some(reason);
    // A stop the user did not ask for is worth announcing.
    if reason != FinishReason::ModelStopped {
        app.notice(format!("run stopped: {}", reason_text(reason)));
    }
}

/// Record an infrastructure failure of the run.
fn failed(app: &mut App, error: VoxError) {
    // The detail is diagnostic and belongs in the log; the user sees
    // the generic message the error type already provides.
    tracing::error!(detail = %error.detail(), "agent run failed");
    app.state = RunState::Idle;
    app.cancelling = false;
    app.notice(error.to_string());
}

/// Human text for why a run ended.
pub fn reason_text(reason: FinishReason) -> &'static str {
    match reason {
        FinishReason::ModelStopped => "the model stopped",
        FinishReason::TurnLimit => "the turn limit was reached",
        FinishReason::ToolErrorLimit => "too many tool failures in a row",
        FinishReason::Cancelled => "cancelled",
    }
}
