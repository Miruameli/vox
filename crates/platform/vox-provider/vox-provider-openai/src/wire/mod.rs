//! Wire types for the OpenAI-compatible chat completions API.

mod chunks;
mod messages;
mod request;
mod response;

pub use chunks::{ChatChunk, ChunkChoice, ChunkDelta, WireUsage};
pub use messages::{role_name, to_wire_messages, WireMessage};
pub use request::{build, ChatRequest, StreamOptions, WireTool};
pub use response::{
    parse_completion, ChatResponse, ResponseChoice, ResponseMessage, WireFunctionCall, WireToolCall,
};

/// Sentinel payload terminating a chat completion stream.
pub const DONE_SENTINEL: &str = "[DONE]";
