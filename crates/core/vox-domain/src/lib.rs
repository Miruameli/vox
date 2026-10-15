//! Pure domain types for conversations, completions, and tools.

pub mod completion;
pub mod message;
pub mod provider;
pub mod tool;

pub use completion::{CompletionRequest, StreamEvent, Usage};
pub use message::{ContentBlock, Message, Role};
pub use provider::ModelProvider;
pub use tool::{ToolCall, ToolDefinition, ToolResult};
