//! Apply-level semantics: byte fidelity around edits, summary
//! accounting, and the no-final-newline edges.

use std::fs;

use tempfile::TempDir;
use vox_engine_edit::{apply_patch, line_hash, EditOp, HashlinePatch, UndoJournal};

fn project(content: &str) -> (TempDir, UndoJournal) {
    let dir = TempDir::new().expect("temp dir");
    fs::write(dir.path().join("file.txt"), content).expect("seed");
    let journal = UndoJournal::open(&dir.path().join("undo.jsonl")).expect("journal");
    (dir, journal)
}

fn read(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join("file.txt")).expect("read")
}

fn ops(ops: Vec<EditOp>) -> HashlinePatch {
    HashlinePatch {
        path: "file.txt".to_string(),
        ops,
    }
}

fn replace(line: usize, original: &str, new_lines: &[&str]) -> EditOp {
    EditOp::Replace {
        line,
        expect_hash: line_hash(original),
        new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
    }
}

fn insert(line: usize, original: &str, new_lines: &[&str]) -> EditOp {
    EditOp::Insert {
        line,
        expect_hash: line_hash(original),
        new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn mixed_terminator_edit_keeps_untouched_endings() {
    let (dir, mut journal) = project("alpha\r\nbeta\ngamma\n");
    let patch = ops(vec![replace(1, "alpha", &["ALPHA"])]);
    let summary = apply_patch(dir.path(), &patch, &mut journal).expect("apply");
    assert_eq!(read(&dir), "ALPHA\r\nbeta\ngamma\n");
    assert_eq!(summary.lines_added, 1);
    assert_eq!(summary.lines_removed, 1);
}

#[test]
fn multi_line_operations_are_counted_exactly() {
    let (dir, mut journal) = project("one\ntwo\nthree\n");
    let patch = ops(vec![
        replace(1, "one", &["uno", "ein"]),
        insert(2, "two", &["dos", "deux"]),
        EditOp::Delete {
            line: 3,
            expect_hash: line_hash("three"),
        },
    ]);
    let summary = apply_patch(dir.path(), &patch, &mut journal).expect("apply");
    assert_eq!(read(&dir), "uno\nein\ntwo\ndos\ndeux\n");
    assert_eq!(summary.lines_added, 4);
    assert_eq!(summary.lines_removed, 2);
}

#[test]
fn insert_after_final_line_without_newline_does_not_fuse() {
    let (dir, mut journal) = project("alpha");
    let patch = ops(vec![insert(1, "alpha", &["beta"])]);
    apply_patch(dir.path(), &patch, &mut journal).expect("apply");
    assert_eq!(read(&dir), "alpha\nbeta");
}

#[test]
fn replace_final_line_without_newline_keeps_no_final_newline() {
    let (dir, mut journal) = project("alpha");
    let patch = ops(vec![replace(1, "alpha", &["one", "two"])]);
    apply_patch(dir.path(), &patch, &mut journal).expect("apply");
    assert_eq!(read(&dir), "one\ntwo");
}

#[test]
fn edit_site_borrows_a_crlf_ending() {
    let (dir, mut journal) = project("a\r\nb\n");
    let patch = ops(vec![replace(1, "a", &["x", "y"])]);
    apply_patch(dir.path(), &patch, &mut journal).expect("apply");
    assert_eq!(read(&dir), "x\r\ny\r\nb\n");
}
