//! Append-only undo journal (JSONL) of edit pre-images.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use vox_shared::{VoxError, VoxResult};

/// One journal record: the pre-image of one applied edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UndoRecord {
    /// Unix epoch milliseconds when the edit was applied.
    pub timestamp_ms: u64,
    /// File the edit touched.
    pub path: PathBuf,
    /// Pre-image, or `None` when the edit created the file.
    pub pre_image: Option<String>,
}

/// Append-only JSONL undo journal.
///
/// Records are appended and synced on every edit, so a crash loses no
/// committed undo step. The file is never rewritten: rollback replays
/// records in reverse order.
pub struct UndoJournal {
    file: File,
}

impl UndoJournal {
    /// Open (creating if needed) the journal at `path`.
    pub fn open(path: &Path) -> VoxResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;
        Ok(Self { file })
    }

    /// Append the pre-image of one applied edit.
    pub fn record_edit(&mut self, path: &Path, pre_image: Option<String>) -> VoxResult<()> {
        let record = UndoRecord {
            timestamp_ms: now_millis(),
            path: path.to_path_buf(),
            pre_image,
        };
        let line = serde_json::to_string(&record).map_err(|error| VoxError::Edit {
            detail: format!("journal record encode failed: {error}"),
        })?;
        writeln!(self.file, "{line}")?;
        self.file.sync_all()?;
        Ok(())
    }

    /// Read every record, oldest first.
    pub fn records(&mut self) -> VoxResult<Vec<UndoRecord>> {
        self.file.seek(SeekFrom::Start(0))?;
        let reader = BufReader::new(&self.file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let record = serde_json::from_str(&line).map_err(|error| VoxError::Edit {
                detail: format!("journal record decode failed: {error}"),
            })?;
            records.push(record);
        }
        Ok(records)
    }
}

/// Current time as Unix epoch milliseconds.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_journal(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("vox-edit-journal-{name}.jsonl"));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn record_and_read_back_preserves_pre_image() {
        let path = temp_journal("round-trip");
        let mut journal = UndoJournal::open(&path).expect("open");
        journal
            .record_edit(Path::new("/tmp/x.txt"), Some("old\n".to_string()))
            .expect("record");
        let records = journal.records().expect("read back");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].path, PathBuf::from("/tmp/x.txt"));
        assert_eq!(records[0].pre_image.as_deref(), Some("old\n"));
    }

    #[test]
    fn journal_is_append_only_across_reopens() {
        let path = temp_journal("append");
        {
            let mut journal = UndoJournal::open(&path).expect("open");
            journal
                .record_edit(Path::new("a.txt"), None)
                .expect("create");
        }
        {
            let mut journal = UndoJournal::open(&path).expect("reopen");
            journal
                .record_edit(Path::new("b.txt"), Some("v1".to_string()))
                .expect("edit");
            let records = journal.records().expect("read back");
            assert_eq!(records.len(), 2);
            assert_eq!(records[0].pre_image, None);
            assert_eq!(records[0].path, PathBuf::from("a.txt"));
            assert_eq!(records[1].pre_image.as_deref(), Some("v1"));
        }
    }

    #[test]
    fn corrupted_journal_is_rejected_loudly() {
        let path = temp_journal("corrupt");
        std::fs::write(&path, "not json\n").expect("seed");
        let mut journal = UndoJournal::open(&path).expect("open");
        assert!(journal.records().is_err());
    }
}
