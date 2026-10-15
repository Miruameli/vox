//! The interactive terminal UI of Vox (ADR-0012).
//!
//! Three parts, in order of dependency:
//!
//! - [`app`] is the state machine. It knows keys, transcript entries,
//!   and agent events, and nothing about terminals or providers, so
//!   every transition is testable without a tty.
//! - [`render`] turns that state into one frame. It knows terminals
//!   and nothing about the model, so a different surface could render
//!   the same state.
//! - [`run`], [`terminal`], and [`worker`] are the shell: they own the
//!   alternate screen, poll the keyboard, and keep the agent on a task
//!   of its own so a slow turn never freezes the terminal.
//!
//! The composition root - the CLI - resolves configuration, reads the
//! credential, builds the provider, the harness, and the session, and
//! hands the assembled [`Agent`](vox_agent::Agent) to [`run`] with a
//! [`Header`] describing the host. This crate never reads the
//! environment or the configuration file.
//!
//! ```
//! use vox_presentation_tui::Header;
//!
//! let header = Header {
//!     model: "gpt-4o-mini".to_string(),
//!     session_id: "tui-1".to_string(),
//!     project_root: "/repo".into(),
//! };
//! assert_eq!(header.model, "gpt-4o-mini");
//! ```

pub mod app;
pub mod input;
pub mod render;
pub mod run;
mod terminal;
mod worker;

pub use crate::app::{Action, App, AppMsg, Command, Entry, EntryKind, Parsed, Transcript};
pub use crate::input::{InputBuffer, Recall};
pub use crate::run::run;
pub use crate::worker::Worker;

/// Facts the surface shows about its host.
///
/// The composition root fills these in, because the presentation layer
/// must never read the environment or the configuration file itself.
#[derive(Debug, Clone)]
pub struct Header {
    /// Model identifier in use.
    pub model: String,
    /// Session id whose file receives the durable events.
    pub session_id: String,
    /// Project root the tools resolve against.
    pub project_root: std::path::PathBuf,
}
