//! Tool harness: the model-facing file operations of Vox.
//!
//! Three tools sit on top of the engines in `vox-engine-edit`:
//!
//! - [`read`]: a hashed, paged view of a file, so the model addresses
//!   lines by content hash instead of retyping context.
//! - [`edit`]: a hashline patch applied atomically, with the whole
//!   patch rejected when any anchor no longer matches (ADR-0005).
//! - [`write`]: a structured whole-file creation or rewrite.
//!
//! Every tool speaks [`ToolCall`](vox_domain::ToolCall) in and
//! [`ToolResult`](vox_domain::ToolResult) out. Expected failures are
//! values: the result carries `is_error` and a message the model can
//! act on. Only infrastructure failures propagate as [`VoxError`], and
//! their diagnostic detail is for the log, never the result text.

mod edit;
mod harness;
mod read;
mod write;

pub use edit::EDIT_TOOL;
pub use harness::ToolHarness;
pub use read::{hashed_view_range, READ_TOOL, READ_VIEW_LIMIT};
pub use write::WRITE_TOOL;

/// Every tool this harness dispatches, for registration with providers.
pub fn definitions() -> Vec<vox_domain::ToolDefinition> {
    vec![read::definition(), edit::definition(), write::definition()]
}
