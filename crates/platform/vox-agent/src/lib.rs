//! The agent runtime: a bounded, durable tool-use loop (ADR-0009).
//!
//! One run is one prompt. The loop sends the conversation plus the tool
//! definitions to the provider, executes every requested tool through
//! [`ToolHarness`], and appends every message, tool call, and tool
//! result to the durable session before the next request. Bounds
//! ([`Limits`]) cap turns and consecutive tool failures; a shared
//! cancellation flag stops the run between actions.
//!
//! The loop is UI-agnostic: progress arrives at a sink as
//! [`LoopEvent`]s. Resume rebuilds the conversation from the session
//! file with [`messages_from_events`]. Two entry points exist:
//! [`Agent::run`] for one prompt over an empty conversation, and
//! [`Agent::continue_conversation`] for a surface that keeps the
//! conversation across prompts - the TUI.

mod agent;
mod contract;
mod resume;
mod run_loop;

pub use agent::Agent;
pub use contract::{FinishReason, Limits, LoopEvent, TOOL_OUTPUT_PREVIEW};
pub use resume::{messages_from_events, tool_result_message};
