//! Append-only writer for one session file.

use std::fs::OpenOptions;
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::path::Path;

use vox_shared::{VoxError, VoxResult};

use crate::event::{EventKind, SessionEvent};

/// Appends events to one session file, syncing each write.
///
/// The file is append-only and every append is exactly one line with
/// the next sequence number, so the sequence equals the line number.
/// A trailing partial line is a write the crash lost: it never
/// committed, and its sequence number is reused.
pub struct SessionWriter {
    file: std::fs::File,
    next_seq: u64,
}

impl SessionWriter {
    /// Open (creating if needed) the session file at `path`.
    pub fn open(path: &Path) -> VoxResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;
        let complete_lines = complete_line_count(&mut file)?;
        Ok(Self {
            file,
            next_seq: complete_lines + 1,
        })
    }

    /// Append one event, redacted and synced, and return it as stored.
    pub fn append(&mut self, kind: EventKind) -> VoxResult<SessionEvent> {
        let mut event = SessionEvent::now(self.next_seq, kind);
        event.redact();
        let line = serde_json::to_string(&event).map_err(|error| VoxError::Config {
            detail: format!("session event encode failed: {error}"),
        })?;
        writeln!(self.file, "{line}")?;
        self.file.sync_all()?;
        self.next_seq += 1;
        Ok(event)
    }
}

/// Count newline-terminated lines of the session file.
///
/// A trailing segment without its newline is a write the crash lost:
/// it never committed and its sequence number is reusable.
fn complete_line_count(file: &mut std::fs::File) -> VoxResult<u64> {
    use std::io::Read;
    file.seek(SeekFrom::Start(0))?;
    let mut text = String::new();
    BufReader::new(&*file).read_to_string(&mut text)?;
    file.seek(SeekFrom::End(0))?;
    let count = text
        .split_inclusive('\n')
        .filter(|segment| segment.ends_with('\n') && !segment.trim().is_empty())
        .count();
    Ok(count as u64)
}
