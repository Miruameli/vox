//! Line-addressed patch types and whole-patch validation.

use serde::Deserialize;

/// Hex alphabet accepted in anchor hashes.
const HEX: &[u8] = b"0123456789abcdef";

/// One line-level operation anchored by content hash.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditOp {
    /// Replace the anchored line with `new_lines`.
    Replace {
        /// 1-based line number of the anchor.
        line: usize,
        /// Expected content hash of the anchor line.
        expect_hash: String,
        /// Replacement lines, applied in place of the anchor.
        new_lines: Vec<String>,
    },
    /// Insert `new_lines` immediately after the anchored line.
    Insert {
        /// 1-based line number of the anchor.
        line: usize,
        /// Expected content hash of the anchor line.
        expect_hash: String,
        /// Lines inserted after the anchor.
        new_lines: Vec<String>,
    },
    /// Delete the anchored line.
    Delete {
        /// 1-based line number of the anchor.
        line: usize,
        /// Expected content hash of the anchor line.
        expect_hash: String,
    },
}

impl EditOp {
    /// `(line number, expected hash)` of the anchor line.
    pub fn anchor(&self) -> (usize, &str) {
        let (EditOp::Replace {
            line, expect_hash, ..
        }
        | EditOp::Insert {
            line, expect_hash, ..
        }
        | EditOp::Delete {
            line, expect_hash, ..
        }) = self;
        (*line, expect_hash)
    }

    pub(crate) fn anchor_line(&self) -> usize {
        self.anchor().0
    }

    fn new_lines(&self) -> &[String] {
        match self {
            Self::Replace { new_lines, .. } | Self::Insert { new_lines, .. } => new_lines,
            Self::Delete { .. } => &[],
        }
    }
}

/// A patch against a single file, applied atomically or rejected whole.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HashlinePatch {
    /// Path of the target file relative to the project root.
    pub path: String,
    /// Operations; each anchors a distinct line.
    pub ops: Vec<EditOp>,
}

impl HashlinePatch {
    /// Validate the patch against the target's line count.
    ///
    /// Every failure mode rejects the whole patch before the file is
    /// touched: no ops, duplicate or out-of-range anchors, malformed
    /// hashes, and replacement lines carrying interior terminators.
    pub fn validate(&self, line_count: usize) -> Result<(), String> {
        if self.ops.is_empty() {
            return Err("patch carries no operations".to_string());
        }
        let mut seen: Vec<usize> = Vec::with_capacity(self.ops.len());
        for op in &self.ops {
            let (line, expect_hash) = op.anchor();
            if line == 0 || line > line_count {
                return Err(format!("line {line} is outside 1..={line_count}"));
            }
            if seen.contains(&line) {
                return Err(format!("line {line} is anchored twice"));
            }
            seen.push(line);
            if expect_hash.len() != crate::HASH_LEN
                || !expect_hash.bytes().all(|byte| HEX.contains(&byte))
            {
                return Err(format!("line {line} has a malformed anchor hash"));
            }
            if op.new_lines().is_empty() && !matches!(op, EditOp::Delete { .. }) {
                return Err(format!("line {line} replaces with no lines"));
            }
            if op
                .new_lines()
                .iter()
                .any(|line| line.contains('\n') || line.contains('\r'))
            {
                return Err(format!("line {line} carries an interior line terminator"));
            }
        }
        Ok(())
    }
}
