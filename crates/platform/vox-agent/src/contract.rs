//! The loop's contract: bounds, outcome, and progress events.
//!
//! These types are the whole public surface a caller of the loop needs,
//! and they travel together: `Limits` decides when a run ends,
//! [`FinishReason`] reports how it ended, and [`LoopEvent`] reports what
//! happened on the way. They live in one file because they are one
//! contract - splitting them would spread a single concept over three
//! headers. The event shape is deliberately small: a UI can render a
//! run from these events alone, and a test can script one.

/// Bounds for one agent run.
///
/// The mandate forbids runaway agents: every loop has a hard turn cap
/// and a cap on consecutive tool failures, and both are part of the
/// public contract so callers can tune or audit them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum provider round trips per run.
    pub max_turns: usize,
    /// Maximum consecutive tool failures before the run stops.
    pub max_consecutive_tool_errors: usize,
}

impl Limits {
    /// Build limits; both values must be at least one.
    pub fn new(max_turns: usize, max_consecutive_tool_errors: usize) -> Self {
        Self {
            max_turns: max_turns.max(1),
            max_consecutive_tool_errors: max_consecutive_tool_errors.max(1),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::new(25, 3)
    }
}

/// Why a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    /// The model stopped without requesting more tools.
    ModelStopped,
    /// The turn cap was reached.
    TurnLimit,
    /// Consecutive tool failures reached the cap.
    ToolErrorLimit,
    /// The run was cancelled between actions.
    Cancelled,
}

/// Longest tool output carried in an event.
///
/// A tool result can be thousands of lines (a read returns a whole
/// page). Carrying all of it to a UI would make every event unbounded,
/// so the loop sends at most this many characters and appends a marked
/// note saying how much was left out. The complete output stays in the
/// session file, which is the durable record.
pub const TOOL_OUTPUT_PREVIEW: usize = 240;

/// Progress events emitted while a run executes.
///
/// The sink is a plain callback, so the loop stays UI-agnostic: the
/// terminal UI, a test, or a headless runner all consume the same
/// stream.
#[derive(Debug, Clone, PartialEq)]
pub enum LoopEvent {
    /// A provider round trip started.
    TurnStarted {
        /// 1-based turn number.
        turn: usize,
    },
    /// The model produced text output.
    Text {
        /// The text produced this turn.
        text: String,
    },
    /// A tool call started executing.
    ToolStarted {
        /// Tool name.
        name: String,
        /// Bounded preview of the call arguments, already marked if
        /// truncated.
        detail: String,
    },
    /// A tool call finished.
    ToolFinished {
        /// Tool name.
        name: String,
        /// Whether the tool reported failure.
        is_error: bool,
        /// Bounded preview of the tool output, already marked if
        /// truncated.
        detail: String,
    },
    /// The run ended.
    Finished {
        /// Why the run ended.
        reason: FinishReason,
    },
}

/// Bound one event payload, marking what was left out.
///
/// The marker names the hidden length, so no output disappears
/// silently: a reader knows how much it is not seeing.
pub(crate) fn preview(text: &str) -> String {
    if text.chars().count() <= TOOL_OUTPUT_PREVIEW {
        return text.to_string();
    }
    let shown: String = text.chars().take(TOOL_OUTPUT_PREVIEW).collect();
    let hidden = text.chars().count() - TOOL_OUTPUT_PREVIEW;
    format!("{shown}... [{hidden} more characters not shown]")
}

/// One-line rendering of tool arguments for an event payload.
pub(crate) fn arguments_preview(arguments: &serde_json::Value) -> String {
    let text = match arguments {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(key, value)| match value {
                serde_json::Value::String(text) => format!("{key}: {text}"),
                other => format!("{key}: {other}"),
            })
            .collect::<Vec<_>>()
            .join(" "),
        other => other.to_string(),
    };
    preview(&text)
}
