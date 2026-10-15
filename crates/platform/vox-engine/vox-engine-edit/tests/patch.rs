//! Validation and text-model tests for the patch module: every failure
//! mode rejects the whole patch, and split/join round-trips byte for
//! byte across LF, CRLF, and missing final newline.

use vox_engine_edit::{join_lines, split_lines, EditOp, HashlinePatch, HASH_LEN};

fn replace(line: usize, hash: &str, new_lines: &[&str]) -> EditOp {
    EditOp::Replace {
        line,
        expect_hash: hash.to_string(),
        new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
    }
}

fn patch(ops: Vec<EditOp>) -> HashlinePatch {
    HashlinePatch {
        path: "src/main.rs".to_string(),
        ops,
    }
}

#[test]
fn empty_patch_is_rejected() {
    assert!(patch(vec![]).validate(10).is_err());
}

#[test]
fn out_of_range_and_duplicate_anchors_are_rejected() {
    let hash = &"a".repeat(HASH_LEN);
    assert!(patch(vec![replace(11, hash, &["x"])]).validate(10).is_err());
    assert!(patch(vec![replace(0, hash, &["x"])]).validate(10).is_err());
    assert!(
        patch(vec![replace(2, hash, &["x"]), replace(2, hash, &["y"])])
            .validate(10)
            .is_err()
    );
}

#[test]
fn malformed_hash_and_interior_terminator_are_rejected() {
    assert!(patch(vec![replace(1, "short", &["x"])])
        .validate(10)
        .is_err());
    assert!(patch(vec![replace(1, &"A".repeat(HASH_LEN), &["x"])])
        .validate(10)
        .is_err());
    assert!(patch(vec![replace(1, &"a".repeat(HASH_LEN), &["a\nb"])])
        .validate(10)
        .is_err());
}

#[test]
fn valid_patch_passes_validation() {
    let hash = line_hash_fixture();
    let valid = patch(vec![
        replace(1, &hash, &["x"]),
        EditOp::Insert {
            line: 2,
            expect_hash: hash.clone(),
            new_lines: vec!["y".to_string()],
        },
        EditOp::Delete {
            line: 3,
            expect_hash: hash,
        },
    ]);
    assert!(valid.validate(10).is_ok());
}

fn line_hash_fixture() -> String {
    "a".repeat(HASH_LEN)
}

#[test]
fn delete_needs_no_replacement_lines() {
    let patch = patch(vec![EditOp::Delete {
        line: 3,
        expect_hash: "a".repeat(HASH_LEN),
    }]);
    assert!(patch.validate(10).is_ok());
}

#[test]
fn split_and_join_round_trip_preserves_bytes() {
    for text in [
        "a\nb\nc\n",
        "a\r\nb\r\nc\r\n",
        "a\nb\nc",
        "a\r\nb\nc",
        "",
        "\n",
        "a\n\nb\n",
    ] {
        assert_eq!(join_lines(&split_lines(text)), text);
    }
}

#[test]
fn mixed_terminators_are_carried_per_line() {
    let lines = split_lines("a\r\nb\nc");
    let endings: Vec<&str> = lines.iter().map(|line| line.terminator).collect();
    assert_eq!(endings, vec!["\r\n", "\n", ""]);
    assert_eq!(join_lines(&lines), "a\r\nb\nc");
}
