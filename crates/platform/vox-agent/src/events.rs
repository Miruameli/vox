//! Progress events a UI sink receives from the loop.

use crate::bounds::FinishReason;

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
    },
    /// A tool call finished.
    ToolFinished {
        /// Tool name.
        name: String,
        /// Whether the tool reported failure.
        is_error: bool,
    },
    /// The run ended.
    Finished {
        /// Why the run ended.
        reason: FinishReason,
    },
}
