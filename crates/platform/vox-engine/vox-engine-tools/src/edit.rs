//! Edit tool: apply a hashline patch through the edit engine.

use vox_domain::ToolResult;
use vox_engine_edit::{apply_patch, HashlinePatch};

use crate::harness::ToolHarness;
use crate::read::parse;

/// Tool name as referenced by the model.
pub const EDIT_TOOL: &str = "edit";

/// The provider-facing description of the edit tool.
pub fn definition() -> vox_domain::ToolDefinition {
    vox_domain::ToolDefinition {
        name: EDIT_TOOL.to_string(),
        description:
            "Edit a file with line-anchored operations. Every op addresses a line by its content hash from a previous read; any mismatch rejects the whole patch."
                .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "ops": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "op": {"enum": ["replace", "insert", "delete"]},
                            "line": {"type": "integer", "minimum": 1},
                            "expect_hash": {"type": "string"},
                            "new_lines": {"type": "array", "items": {"type": "string"}},
                        },
                        "required": ["op", "line", "expect_hash"],
                    },
                },
            },
            "required": ["path", "ops"],
        }),
    }
}

impl ToolHarness {
    /// Execute an edit call.
    ///
    /// Argument and anchor failures are tool-level failures: the file is
    /// left untouched and the result message names what the model must
    /// fix - the decoder detail or the rejected anchor.
    pub(crate) fn edit(
        &mut self,
        call: &vox_domain::ToolCall,
    ) -> vox_shared::VoxResult<ToolResult> {
        let patch = match parse::<HashlinePatch>(call) {
            Ok(patch) => patch,
            Err(detail) => return Ok(ToolResult::failure(&call.id, detail)),
        };
        let root = self.root().to_path_buf();
        let summary = match apply_patch(&root, &patch, self.journal_mut()) {
            Ok(summary) => summary,
            Err(error) => return crate::harness::rejection(call, error),
        };
        Ok(ToolResult::success(
            &call.id,
            format!(
                "edited {}: +{} -{} lines",
                patch.path, summary.lines_added, summary.lines_removed
            ),
        ))
    }
}
