//! Glob tool: path pattern matching over the project tree.

use serde::Deserialize;
use vox_domain::{ToolCall, ToolDefinition, ToolResult};
use vox_engine_edit::resolve_within;
use vox_shared::VoxResult;

use crate::glob_pattern::matches;
use crate::harness::{rejection, ToolHarness};
use crate::tools::read::parse;
use crate::walk;

/// Tool name as referenced by the model.
pub const GLOB_TOOL: &str = "glob";

/// Default and maximum number of paths one call returns.
pub const GLOB_PATH_LIMIT: usize = 200;

/// Arguments of a glob call.
#[derive(Debug, Deserialize)]
pub struct GlobArgs {
    /// Pattern with `/`-separated segments; `*` and `?` stay inside
    /// one segment, `**` spans any number of directories.
    pub pattern: String,
    /// Directory to search, relative to the project root; defaults
    /// to the whole project.
    #[serde(default)]
    pub path: Option<String>,
    /// Paths to return; defaults to [`GLOB_PATH_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// The provider-facing description of the glob tool.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: GLOB_TOOL.to_string(),
        description: "List project files whose path matches a pattern. \
                      Segments support * and ?; ** spans directories. \
                      Paths are returned sorted."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Pattern like crates/*/src or **/*.rs."
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search, relative to the project root."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum paths to return."
                }
            },
            "required": ["pattern"]
        }),
    }
}

impl ToolHarness {
    /// Match project paths against a pattern.
    pub(crate) fn glob(&mut self, call: &ToolCall) -> VoxResult<ToolResult> {
        let args: GlobArgs = match parse(call) {
            Ok(args) => args,
            Err(detail) => return Ok(ToolResult::failure(&call.id, detail)),
        };
        let scope = match &args.path {
            Some(path) => match resolve_within(self.root(), path) {
                Ok(scope) => scope,
                Err(error) => return rejection(call, error),
            },
            None => self.root().to_path_buf(),
        };
        if !scope.is_dir() {
            let target = args.path.as_deref().unwrap_or(".");
            return Ok(ToolResult::failure(
                &call.id,
                format!("no such directory {target}"),
            ));
        }

        let mut hits: Vec<String> = Vec::new();
        for file in walk::files(&scope) {
            let shown = file.strip_prefix(self.root()).unwrap_or(&file);
            let relative = shown.to_string_lossy().into_owned();
            if matches(&args.pattern, &relative) {
                hits.push(relative);
            }
        }
        hits.sort();

        let limit = args.limit.unwrap_or(GLOB_PATH_LIMIT).min(GLOB_PATH_LIMIT);
        let hidden = hits.len().saturating_sub(limit);
        hits.truncate(limit);
        if hidden > 0 {
            hits.push(format!("... {hidden} more paths"));
        }
        let text = if hits.is_empty() {
            "no paths".to_string()
        } else {
            hits.join("\n")
        };
        Ok(ToolResult::success(&call.id, text))
    }
}
