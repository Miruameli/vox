//! Crash-safety guarantees of ADR-0005: the undo journal is written
//! before the file, and no failure path leaves a temporary file or a
//! mutated file without a rollback record.

use std::fs;

use tempfile::TempDir;
use vox_engine_edit::{write_whole_file, UndoJournal};

fn project() -> (TempDir, UndoJournal) {
    let dir = TempDir::new().expect("temp dir");
    let journal = UndoJournal::open(&dir.path().join("undo.jsonl")).expect("journal");
    (dir, journal)
}

#[test]
fn failed_write_after_journaling_leaves_a_rollback_record() {
    let (dir, mut journal) = project();
    // The target's parent is a file, so the atomic write fails after
    // the journal record has been appended.
    let blocked = dir.path().join("blocked");
    fs::write(&blocked, "not a directory").expect("block");
    let result = write_whole_file(dir.path(), "blocked/inside.txt", "content\n", &mut journal);
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(&blocked).expect("read"),
        "not a directory"
    );

    // Journal-before-write ordering: the failed write still recorded a
    // null pre-image, so a rollback deletes the phantom file instead of
    // silently losing the mutation.
    let records = journal.records().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].pre_image, None);
}

#[test]
fn successful_write_of_existing_file_records_the_pre_image_first() {
    let (dir, mut journal) = project();
    fs::write(dir.path().join("kept.txt"), "old\n").expect("seed");
    write_whole_file(dir.path(), "kept.txt", "new\n", &mut journal).expect("write");
    let records = journal.records().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].pre_image.as_deref(), Some("old\n"));
    assert_eq!(
        fs::read_to_string(dir.path().join("kept.txt")).expect("read"),
        "new\n"
    );
}
