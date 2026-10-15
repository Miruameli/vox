//! Engine-level behaviour required by ADR-0005: valid anchors apply
//! exactly, stale anchors reject the whole patch with the file intact,
//! and whole-file writes are atomic with a journaled pre-image.

use std::fs;

use tempfile::TempDir;
use vox_engine_edit::{
    apply_patch, line_hash, write_whole_file, EditOp, HashlinePatch, UndoJournal,
};

fn project(content: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("main.rs");
    fs::write(&path, content).expect("seed file");
    (dir, path)
}

fn journal(dir: &TempDir) -> UndoJournal {
    UndoJournal::open(&dir.path().join("undo.jsonl")).expect("journal")
}

fn replace(line: usize, content: &str, new_lines: &[&str]) -> EditOp {
    EditOp::Replace {
        line,
        expect_hash: line_hash(content),
        new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn valid_patch_applies_exactly() {
    let (dir, path) = project("fn one() {}\nfn two() {}\nfn three() {}\n");
    let patch = HashlinePatch {
        path: "main.rs".to_string(),
        ops: vec![replace(2, "fn two() {}", &["fn TWO() {}"])],
    };
    let mut undo = journal(&dir);
    let summary = apply_patch(dir.path(), &patch, &mut undo).expect("apply");
    assert_eq!(summary.lines_added, 1);
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "fn one() {}\nfn TWO() {}\nfn three() {}\n"
    );
    let records = undo.records().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].pre_image.as_deref(),
        Some("fn one() {}\nfn two() {}\nfn three() {}\n")
    );
}

#[test]
fn stale_anchor_rejects_whole_patch_and_leaves_file_intact() {
    let (dir, path) = project("alpha\nbeta\ngamma\n");
    let patch = HashlinePatch {
        path: "main.rs".to_string(),
        ops: vec![
            // Line 1 anchor is valid, line 3 anchor is stale.
            EditOp::Insert {
                line: 1,
                expect_hash: line_hash("alpha"),
                new_lines: vec!["zero".to_string()],
            },
            EditOp::Delete {
                line: 3,
                expect_hash: line_hash("not-gamma"),
            },
        ],
    };
    let mut undo = journal(&dir);
    let error = apply_patch(dir.path(), &patch, &mut undo).expect_err("stale anchor");
    assert!(error.to_string().contains("edit rejected"));
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "alpha\nbeta\ngamma\n"
    );
    assert!(undo.records().expect("records").is_empty());
}

#[test]
fn insert_and_delete_apply_from_one_patch() {
    let (dir, path) = project("keep\nreplace-me\ntail\n");
    let patch = HashlinePatch {
        path: "main.rs".to_string(),
        ops: vec![
            replace(2, "replace-me", &["replacement"]),
            EditOp::Insert {
                line: 3,
                expect_hash: line_hash("tail"),
                new_lines: vec!["added".to_string()],
            },
        ],
    };
    let mut undo = journal(&dir);
    apply_patch(dir.path(), &patch, &mut undo).expect("apply");
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "keep\nreplacement\ntail\nadded\n"
    );
}

#[test]
fn path_traversal_outside_the_project_is_rejected() {
    let (dir, _) = project("x\n");
    let patch = HashlinePatch {
        path: "../escape.txt".to_string(),
        ops: vec![replace(1, "x", &["y"])],
    };
    let mut undo = journal(&dir);
    assert!(apply_patch(dir.path(), &patch, &mut undo).is_err());
    assert!(apply_patch(dir.path(), &patch, &mut undo).is_err());
}

#[test]
fn whole_file_write_is_atomic_and_journals_the_pre_image() {
    let (dir, path) = project("old\n");
    let mut undo = journal(&dir);
    let summary = write_whole_file(dir.path(), "main.rs", "fresh\n", &mut undo).expect("write");
    assert!(!summary.created);
    assert_eq!(fs::read_to_string(&path).expect("read"), "fresh\n");
    let records = undo.records().expect("records");
    assert_eq!(records[0].pre_image.as_deref(), Some("old\n"));

    let created =
        write_whole_file(dir.path(), "new.txt", "brand new\n", &mut undo).expect("create");
    assert!(created.created);
    assert_eq!(
        fs::read_to_string(dir.path().join("new.txt")).expect("read"),
        "brand new\n"
    );
    let records = undo.records().expect("records");
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].pre_image, None);
}

#[test]
fn missing_target_reports_a_rejected_edit() {
    let dir = TempDir::new().expect("temp dir");
    let patch = HashlinePatch {
        path: "absent.txt".to_string(),
        ops: vec![replace(1, "x", &["y"])],
    };
    let mut undo = journal(&dir);
    assert!(apply_patch(dir.path(), &patch, &mut undo).is_err());
}
