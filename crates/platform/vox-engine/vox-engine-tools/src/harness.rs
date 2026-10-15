//! The tool harness: one dispatch point over read, edit, and write.

use std::path::{Path, PathBuf};

use vox_domain::{ToolCall, ToolResult};
use vox_engine_edit::{UndoJournal, UndoRecord};
use vox_shared::{VoxError, VoxResult};

/// Dispatches tool calls against a project root with one undo journal.
pub struct ToolHarness {
    root: PathBuf,
    journal: UndoJournal,
}

impl ToolHarness {
    /// Open a harness rooted at `root` with its journal at `journal_path`.
    pub fn new(root: impl Into<PathBuf>, journal_path: &Path) -> VoxResult<Self> {
        let journal = UndoJournal::open(journal_path)?;
        Ok(Self {
            root: root.into(),
            journal,
        })
    }

    /// The project root every tool resolves against.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Execute one tool call.
    ///
    /// Argument shape failures propagate as [`VoxError::Edit`] so the
    /// caller can log the detail; the message the model sees stays
    /// generic.
    pub fn call(&mut self, call: &ToolCall) -> VoxResult<ToolResult> {
        match call.name.as_str() {
            crate::tools::read::READ_TOOL => self.read(call),
            crate::tools::edit::EDIT_TOOL => self.edit(call),
            crate::tools::write::WRITE_TOOL => self.write(call),
            crate::tools::grep::GREP_TOOL => self.grep(call),
            crate::tools::glob::GLOB_TOOL => self.glob(call),
            crate::tools::shell::bash::BASH_TOOL => self.bash(call),
            other => Ok(ToolResult::failure(
                &call.id,
                format!("unknown tool {other}"),
            )),
        }
    }

    /// The undo journal records of every applied mutation.
    pub fn undo_records(&mut self) -> VoxResult<Vec<UndoRecord>> {
        self.journal.records()
    }

    pub(crate) fn journal_mut(&mut self) -> &mut UndoJournal {
        &mut self.journal
    }
}

/// Map an engine rejection to a model-facing tool failure.
///
/// Engine rejections (patch validation, anchor mismatch, path safety)
/// carry details the model already knows - it sent the patch - so the
/// detail is the actionable message: which line, which file, why. Any
/// other error stays an infrastructure failure for the caller to log.
pub(crate) fn rejection(call: &ToolCall, error: VoxError) -> VoxResult<ToolResult> {
    match error {
        VoxError::Edit { detail } => Ok(ToolResult::failure(&call.id, detail)),
        other => Err(other),
    }
}
