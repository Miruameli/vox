//! Edit engine: hashline patches, atomic writes, and an undo journal.
//!
//! The engine applies model-authored edits in the two modes decided by
//! ADR-0005:
//!
//! 1. [`HashlinePatch`]: every target line is anchored by its content
//!    hash. Anchors are verified against the current file before any
//!    mutation; a single mismatch rejects the whole patch.
//! 2. [`write_whole_file`]: structured creation or full rewrite of a
//!    file, applied atomically.
//!
//! Both modes write through a temporary file in the target directory
//! and rename it into place, so a failed edit leaves the original file
//! byte-for-byte intact, and both record the pre-image in the undo
//! journal.

mod atomic;
mod hashline;
mod patch;
mod undo;

pub use hashline::{anchor_matches, hashed_view, line_hash, HASH_LEN};
pub use patch::{join_lines, split_lines, EditOp, HashlinePatch};
pub use undo::{UndoJournal, UndoRecord};

use std::path::{Path, PathBuf};

use vox_shared::{VoxError, VoxResult};

/// Outcome of one applied edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    /// File the edit touched.
    pub path: PathBuf,
    /// Lines inserted or replaced in.
    pub lines_added: usize,
    /// Lines replaced out or deleted.
    pub lines_removed: usize,
    /// `true` when the edit created the file.
    pub created: bool,
}

/// Resolve a model-supplied path against the project root.
///
/// Absolute paths and any `..` traversal are rejected; the resolved
/// path must stay inside `root`.
pub fn resolve_within(root: &Path, relative: &str) -> VoxResult<PathBuf> {
    let candidate = Path::new(relative);
    if relative.is_empty() || candidate.is_absolute() || candidate.has_root() {
        return Err(VoxError::Edit {
            detail: format!("path {relative:?} is not a relative project path"),
        });
    }
    if candidate
        .components()
        .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(VoxError::Edit {
            detail: format!("path {relative:?} escapes the project root"),
        });
    }
    Ok(root.join(candidate))
}

/// Apply a hashline patch to an existing file.
///
/// The whole patch is rejected, and the file is left untouched, when
/// validation fails or any anchor no longer matches the file content.
/// On success the pre-image is appended to the undo journal.
pub fn apply_patch(
    root: &Path,
    patch: &HashlinePatch,
    undo: &mut UndoJournal,
) -> VoxResult<ApplySummary> {
    let path = resolve_within(root, &patch.path)?;
    let original = std::fs::read_to_string(&path).map_err(|error| VoxError::Edit {
        detail: format!("{}: {error}", patch.path),
    })?;
    let (mut lines, terminator, ends_with_newline) = patch::split_lines(&original);
    patch
        .validate(lines.len())
        .map_err(|detail| VoxError::Edit { detail })?;
    for op in &patch.ops {
        let (line, expect_hash) = op.anchor();
        if !anchor_matches(expect_hash, &lines[line - 1]) {
            return Err(VoxError::Edit {
                detail: format!("{}:{line} anchor mismatch", patch.path),
            });
        }
    }
    let mut ops = patch.ops.clone();
    ops.sort_by_key(|op| std::cmp::Reverse(op.anchor_line()));
    let (mut added, mut removed) = (0usize, 0usize);
    for op in ops {
        match op {
            EditOp::Replace {
                line, new_lines, ..
            } => {
                lines.splice(line - 1..line, new_lines);
                added += 1;
                removed += 1;
            }
            EditOp::Insert {
                line, new_lines, ..
            } => {
                lines.splice(line..line, new_lines);
                added += 1;
            }
            EditOp::Delete { line, .. } => {
                lines.remove(line - 1);
                removed += 1;
            }
        }
    }
    let updated = patch::join_lines(&lines, terminator, ends_with_newline);
    let summary = ApplySummary {
        path: path.clone(),
        lines_added: added,
        lines_removed: removed,
        created: false,
    };
    atomic::write_atomic(&path, &updated)?;
    undo.record_edit(&path, Some(original))?;
    Ok(summary)
}

/// Create or fully replace a file atomically.
///
/// The pre-image of an existing file is appended to the undo journal;
/// creating a file records `None` so a rollback can delete it.
pub fn write_whole_file(
    root: &Path,
    relative: &str,
    content: &str,
    undo: &mut UndoJournal,
) -> VoxResult<ApplySummary> {
    let path = resolve_within(root, relative)?;
    let pre_image = std::fs::read_to_string(&path).ok();
    let created = pre_image.is_none();
    let summary = ApplySummary {
        path: path.clone(),
        lines_added: content.lines().count(),
        lines_removed: pre_image
            .as_ref()
            .map(|old| old.lines().count())
            .unwrap_or(0),
        created,
    };
    atomic::write_atomic(&path, content)?;
    undo.record_edit(&path, pre_image)?;
    Ok(summary)
}
