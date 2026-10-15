//! Write tool: structured whole-file creation or rewrite.

use serde::Deserialize;
use vox_domain::ToolResult;
use vox_engine_edit::write_whole_file;

use crate::harness::ToolHarness;
use crate::read::parse;

/// Tool name as referenced by the model.
pub const WRITE_TOOL: &str = "write";

/// Arguments of a write call.
#[derive(Debug, Deserialize)]
pub struct WriteArgs {
    /// File to create or replace, relative to the project root.
    pub path: String,
    /// Full file content.
    pub content: String,
}

/// The provider-facing description of the write tool.
pub fn definition() -> vox_domain::ToolDefinition {
    vox_domain::ToolDefinition {
        name: WRITE_TOOL.to_string(),
        description:
            "Create or fully replace a file with the given content. Applied atomically; the previous content is journaled for undo."
                .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"},
            },
            "required": ["path", "content"],
        }),
    }
}

impl ToolHarness {
    /// Execute a write call.
    pub(crate) fn write(
        &mut self,
        call: &vox_domain::ToolCall,
    ) -> vox_shared::VoxResult<ToolResult> {
        let args = match parse::<WriteArgs>(call) {
            Ok(args) => args,
            Err(detail) => return Ok(ToolResult::failure(&call.id, detail)),
        };
        let root = self.root().to_path_buf();
        let summary = match write_whole_file(&root, &args.path, &args.content, self.journal_mut()) {
            Ok(summary) => summary,
            Err(error) => return crate::harness::rejection(call, error),
        };
        let verb = if summary.created {
            "created"
        } else {
            "replaced"
        };
        Ok(ToolResult::success(
            &call.id,
            format!("{verb} {} ({} lines)", args.path, summary.lines_added),
        ))
    }
}
