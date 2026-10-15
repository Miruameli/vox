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
//! file with [`messages_from_events`].

mod agent;
mod bounds;
mod events;
mod resume;

pub use agent::Agent;
pub use bounds::{FinishReason, Limits};
pub use events::LoopEvent;
pub use resume::{messages_from_events, tool_result_message};
