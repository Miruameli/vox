//! Wire types for the OpenAI-compatible chat completions API.
//!
//! Internal API: the submodules map domain contracts to wire shapes
//! and back. Only the symbols sibling modules consume are re-exported
//! here; wire types never leave the crate.

mod chunks;
mod messages;
mod request;
mod response;

pub use chunks::{ChatChunk, WireUsage};
pub use request::{build, ChatRequest};
pub use response::parse_completion;

/// Sentinel payload terminating a chat completion stream.
pub const DONE_SENTINEL: &str = "[DONE]";
