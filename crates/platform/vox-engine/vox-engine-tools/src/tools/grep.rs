//! Grep tool: literal substring search over the project tree.

use serde::Deserialize;
use vox_domain::{ToolCall, ToolDefinition, ToolResult};
use vox_engine_edit::resolve_within;
use vox_shared::VoxResult;

use crate::harness::{rejection, ToolHarness};
use crate::tools::read::parse;
use crate::walk;

/// Tool name as referenced by the model.
pub const GREP_TOOL: &str = "grep";

/// Default and maximum number of matching lines one call returns.
///
/// Unlike the read tool, search cannot reject large requests: the
/// model cannot know the match count before asking. The cap stays
/// hard, and truncation is announced in the result itself (ADR-0009).
pub const GREP_MATCH_LIMIT: usize = 100;

/// Arguments of a grep call.
#[derive(Debug, Deserialize)]
pub struct GrepArgs {
    /// Literal substring to find; an empty pattern is rejected
    /// because it would match every line and waste the cap.
    pub pattern: String,
    /// File or directory to search, relative to the project root;
    /// defaults to the whole project.
    #[serde(default)]
    pub path: Option<String>,
    /// Matching lines to return; defaults to [`GREP_MATCH_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// The provider-facing description of the grep tool.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: GREP_TOOL.to_string(),
        description: "Find lines containing a literal substring in project files. \
                      Returns path:line: content, sorted; at most the limit lines."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Literal substring to search for."
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search, relative to the project root."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum matching lines to return."
                }
            },
            "required": ["pattern"]
        }),
    }
}

impl ToolHarness {
    /// Search for a literal substring and answer with matching lines.
    pub(crate) fn grep(&mut self, call: &ToolCall) -> VoxResult<ToolResult> {
        let args: GrepArgs = match parse(call) {
            Ok(args) => args,
            Err(detail) => return Ok(ToolResult::failure(&call.id, detail)),
        };
        if args.pattern.is_empty() {
            return Ok(ToolResult::failure(&call.id, "pattern must not be empty"));
        }
        let scope = match &args.path {
            Some(path) => match resolve_within(self.root(), path) {
                Ok(scope) => scope,
                Err(error) => return rejection(call, error),
            },
            None => self.root().to_path_buf(),
        };
        if !scope.exists() {
            let target = args.path.as_deref().unwrap_or(".");
            return Ok(ToolResult::failure(
                &call.id,
                format!("no such path {target}"),
            ));
        }

        let pattern = &args.pattern;
        let mut hits: Vec<String> = if scope.is_file() {
            file_matches(&scope, pattern, self.root())
        } else {
            let mut hits = Vec::new();
            for file in walk::files(&scope) {
                hits.extend(file_matches(&file, pattern, self.root()));
            }
            hits
        };
        hits.sort();

        let limit = args.limit.unwrap_or(GREP_MATCH_LIMIT).min(GREP_MATCH_LIMIT);
        let hidden = hits.len().saturating_sub(limit);
        hits.truncate(limit);
        if hidden > 0 {
            hits.push(format!("... {hidden} more matches"));
        }
        let text = if hits.is_empty() {
            "no matches".to_string()
        } else {
            hits.join("\n")
        };
        Ok(ToolResult::success(&call.id, text))
    }
}

/// Matching lines of one file as `path:line: content`.
///
/// Unreadable and non-UTF-8 files are skipped: a search answer
/// covers text files, and binary output would corrupt the result.
fn file_matches(file: &std::path::Path, pattern: &str, root: &std::path::Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    let shown = file.strip_prefix(root).unwrap_or(file);
    text.lines()
        .enumerate()
        .filter(|(_, line)| line.contains(pattern))
        .map(|(index, line)| format!("{}:{}:{}", shown.display(), index + 1, line))
        .collect()
}
