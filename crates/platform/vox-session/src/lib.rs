//! Durable session storage: append-only JSONL events (ADR-0006).
//!
//! One session is one file, `<data-dir>/sessions/<session-id>.jsonl`,
//! holding one event per line. Each event carries a monotonically
//! increasing sequence number, a timestamp, and a typed payload:
//! conversation messages, tool calls and results, applied edits,
//! undos, and session metadata.
//!
//! Two guarantees shape the design:
//!
//! - **Crash safety.** Writes are appended with `O_APPEND` and synced;
//!   a reader tolerates one trailing partial line and rejects any
//!   corruption before it.
//! - **No secrets on disk.** Every string leaf of an event is passed
//!   through [`vox_shared::redaction`] before the line is written, so a
//!   credential-shaped tool result never reaches the file.
//!
//! Session files are plain text: users can read, diff, and share them.

mod event;
mod reader;
mod writer;

pub use event::{EventKind, SessionEvent};
pub use reader::replay;
pub use writer::SessionWriter;

use std::path::{Path, PathBuf};

use vox_shared::VoxResult;

/// Characters allowed in a session id used as a file name.
fn is_safe_session_id(session_id: &str) -> bool {
    !session_id.is_empty()
        && session_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Path of the session file for `session_id` under `data_dir`.
///
/// The id is validated as a plain file name, so a crafted id can never
/// escape the sessions directory.
pub fn session_path(data_dir: &Path, session_id: &str) -> VoxResult<PathBuf> {
    if !is_safe_session_id(session_id) {
        return Err(vox_shared::VoxError::Config {
            detail: format!("session id {session_id:?} is not a safe file name"),
        });
    }
    Ok(data_dir
        .join("sessions")
        .join(format!("{session_id}.jsonl")))
}
