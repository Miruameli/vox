//! Read tool: a hashed, paged view of a file for the model.

use serde::Deserialize;
use vox_domain::ToolResult;
use vox_engine_edit::{line_hash, resolve_within};
use vox_shared::VoxResult;

use crate::harness::ToolHarness;

/// Tool name as referenced by the model.
pub const READ_TOOL: &str = "read";

/// Largest page the read tool returns; larger requests are rejected
/// rather than truncated so the model never loses lines silently.
pub const READ_VIEW_LIMIT: usize = 2000;

/// Arguments of a read call.
#[derive(Debug, Deserialize)]
pub struct ReadArgs {
    /// File to read, relative to the project root.
    pub path: String,
    /// 1-based first line of the page; defaults to the file start.
    #[serde(default)]
    pub offset: Option<usize>,
    /// Page size; defaults to [`READ_VIEW_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// The provider-facing description of the read tool.
pub fn definition() -> vox_domain::ToolDefinition {
    vox_domain::ToolDefinition {
        name: READ_TOOL.to_string(),
        description:
            "Read a file as numbered lines prefixed by content hashes. Address lines by their hash when editing."
                .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "offset": {"type": "integer", "minimum": 1},
                "limit": {"type": "integer", "minimum": 1},
            },
            "required": ["path"],
        }),
    }
}

/// Render `lines[start..end]` with 1-based absolute numbers and hashes.
///
/// The format matches [`vox_engine_edit::hashed_view`] exactly; a test
/// in this crate pins the parity. An empty file has no out-of-range
/// offsets: reading it returns an empty view, because the file is
/// readable, not misplaced.
pub fn hashed_view_range(file: &str, offset: usize, limit: usize) -> Result<String, String> {
    let lines: Vec<&str> = file.lines().collect();
    if offset == 0 || offset > lines.len().max(1) {
        return Err(format!("line {offset} is outside 1..={}", lines.len()));
    }
    let end = (offset - 1).saturating_add(limit).min(lines.len());
    let mut view = String::new();
    for (index, line) in lines[offset - 1..end].iter().enumerate() {
        view.push_str(&format!(
            "{:>5}|{}|{line}\n",
            offset + index,
            line_hash(line)
        ));
    }
    Ok(view)
}

impl ToolHarness {
    /// Execute a read call.
    pub(crate) fn read(&self, call: &vox_domain::ToolCall) -> VoxResult<ToolResult> {
        let args = match parse::<ReadArgs>(call) {
            Ok(args) => args,
            Err(detail) => return Ok(ToolResult::failure(&call.id, detail)),
        };
        let limit = args.limit.unwrap_or(READ_VIEW_LIMIT);
        if limit > READ_VIEW_LIMIT {
            return Ok(ToolResult::failure(
                &call.id,
                format!("limit exceeds the {READ_VIEW_LIMIT}-line page cap"),
            ));
        }
        let path = match resolve_within(self.root(), &args.path) {
            Ok(path) => path,
            Err(error) => return crate::harness::rejection(call, error),
        };
        let file = match std::fs::read_to_string(&path) {
            Ok(file) => file,
            // A missing or unreadable file is a tool-level failure: the
            // value carries a message the model can act on, and no
            // filesystem detail leaks into it.
            Err(_) => {
                return Ok(ToolResult::failure(
                    &call.id,
                    format!("cannot read {}", args.path),
                ));
            }
        };
        match hashed_view_range(&file, args.offset.unwrap_or(1), limit) {
            Ok(view) => Ok(ToolResult::success(&call.id, view)),
            Err(detail) => Ok(ToolResult::failure(&call.id, detail)),
        }
    }
}

/// Deserialize call arguments, returning the decoder's detail message.
///
/// A malformed call is the model's mistake, not infrastructure: every
/// tool turns this into a failure VALUE the model can act on.
pub(crate) fn parse<T: serde::de::DeserializeOwned>(
    call: &vox_domain::ToolCall,
) -> Result<T, String> {
    serde_json::from_value(call.arguments.clone()).map_err(|error| error.to_string())
}
