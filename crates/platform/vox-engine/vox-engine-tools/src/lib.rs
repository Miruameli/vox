//! Tool harness: the model-facing file operations of Vox.
//!
//! Six tools sit on top of the engines in `vox-engine-edit`:
//!
//! - [`read`]: a hashed, paged view of a file, so the model addresses
//!   lines by content hash instead of retyping context.
//! - [`edit`]: a hashline patch applied atomically, with the whole
//!   patch rejected when any anchor no longer matches (ADR-0005).
//! - [`write`]: a structured whole-file creation or rewrite.
//! - [`grep`]: literal substring search over the project tree.
//! - [`glob`]: path pattern matching over the project tree.
//! - [`bash`](tools::shell::bash): run one command in the project
//!   root, with a mandatory timeout and capped output (ADR-0011).
//!
//! Every tool speaks [`ToolCall`](vox_domain::ToolCall) in and
//! [`ToolResult`](vox_domain::ToolResult) out. Expected failures are
//! values: the result carries `is_error` and a message the model can
//! act on. Only infrastructure failures propagate as [`VoxError`], and
//! their diagnostic detail is for the log, never the result text.

mod glob_pattern;
mod harness;
mod walk;

mod tools {
    pub(crate) mod edit;
    pub(crate) mod glob;
    pub(crate) mod grep;
    pub(crate) mod read;
    pub(crate) mod write;
    pub(crate) mod shell {
        pub(crate) mod bash;
        pub(crate) mod pipes;
        pub(crate) mod runner;
    }
}

pub use harness::ToolHarness;
pub use tools::edit::EDIT_TOOL;
pub use tools::glob::{GLOB_PATH_LIMIT, GLOB_TOOL};
pub use tools::grep::{GREP_MATCH_LIMIT, GREP_TOOL};
pub use tools::read::{hashed_view_range, READ_TOOL, READ_VIEW_LIMIT};
pub use tools::shell::bash::BASH_OUTPUT_LIMIT;
pub use tools::write::WRITE_TOOL;

/// Every tool this harness dispatches, for registration with providers.
pub fn definitions() -> Vec<vox_domain::ToolDefinition> {
    vec![
        tools::read::definition(),
        tools::edit::definition(),
        tools::write::definition(),
        tools::grep::definition(),
        tools::glob::definition(),
        tools::shell::bash::definition(),
    ]
}
