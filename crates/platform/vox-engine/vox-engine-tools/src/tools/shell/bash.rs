//! Bash tool: run one command with the project root as the working
//! directory (ADR-0011).

use serde::Deserialize;
use vox_domain::{ToolCall, ToolDefinition, ToolResult};
use vox_shared::VoxResult;

use crate::harness::ToolHarness;
use crate::tools::read::parse;
use crate::tools::shell::runner::{self, Outcome};

/// Tool name as referenced by the model.
pub const BASH_TOOL: &str = "bash";

/// Default timeout for one command, in milliseconds.
pub const BASH_TIMEOUT_DEFAULT_MS: u64 = 30_000;

/// Hard ceiling on any requested timeout, in milliseconds.
pub const BASH_TIMEOUT_MAX_MS: u64 = 60_000;

/// Byte cap per output stream; truncation is announced.
pub const BASH_OUTPUT_LIMIT: usize = 16 * 1024;

/// Arguments of a bash call.
#[derive(Debug, Deserialize)]
pub struct BashArgs {
    /// The program and its arguments as an argv array. The harness
    /// spawns it directly: there is no shell, so no quoting rules
    /// and no string interpolation into one.
    pub command: Vec<String>,
    /// Timeout in milliseconds; defaults to
    /// [`BASH_TIMEOUT_DEFAULT_MS`] and is clamped to
    /// [`BASH_TIMEOUT_MAX_MS`].
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// The provider-facing description of the bash tool.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: BASH_TOOL.to_string(),
        description: "Run a command in the project root. Arguments are an \
                      argv array executed without a shell. Returns the exit \
                      code and captured output; output is capped."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Program and arguments, e.g. [\"ls\", \"-la\"]."
                },
                "timeout_ms": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Timeout in milliseconds."
                }
            },
            "required": ["command"]
        }),
    }
}

impl ToolHarness {
    /// Run one command and answer with its exit code and output.
    pub(crate) fn bash(&self, call: &ToolCall) -> VoxResult<ToolResult> {
        let args: BashArgs = match parse(call) {
            Ok(args) => args,
            Err(detail) => return Ok(ToolResult::failure(&call.id, detail)),
        };
        let Some((program, rest)) = args.command.split_first() else {
            return Ok(ToolResult::failure(&call.id, "command must not be empty"));
        };
        let timeout_ms = args
            .timeout_ms
            .unwrap_or(BASH_TIMEOUT_DEFAULT_MS)
            .min(BASH_TIMEOUT_MAX_MS);
        if timeout_ms == 0 {
            return Ok(ToolResult::failure(&call.id, "timeout must be positive"));
        }

        let outcome = runner::run(program, rest, self.root(), timeout_ms);
        match outcome {
            // A program that cannot start is the model's mistake - it
            // named the binary - so the detail is the actionable part.
            Err(_) => Ok(ToolResult::failure(
                &call.id,
                format!("cannot run {program}"),
            )),
            Ok(Outcome::TimedOut) => Ok(ToolResult::failure(
                &call.id,
                format!("timed out after {timeout_ms} ms"),
            )),
            Ok(Outcome::Exited {
                code,
                stdout,
                stderr,
                complete,
            }) => {
                let mut text = format!("exit {code}");
                append_stream(&mut text, "stdout", &stdout);
                append_stream(&mut text, "stderr", &stderr);
                if !complete {
                    // A pipe was still held open when the budget ran
                    // out - usually a background process that
                    // outlived the command. The model must know the
                    // captured streams may be partial.
                    text.push_str("\n... output may be incomplete");
                }
                Ok(ToolResult::success(&call.id, text))
            }
        }
    }
}

/// Append one stream under the byte cap, announcing any cut.
///
/// A partial UTF-8 sequence at the cut is repaired by the lossy
/// conversion; the marker states the exact bytes dropped so the
/// model knows how much it is not seeing.
fn append_stream(text: &mut String, label: &str, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let dropped = bytes.len().saturating_sub(BASH_OUTPUT_LIMIT);
    let shown = &bytes[..bytes.len().min(BASH_OUTPUT_LIMIT)];
    text.push('\n');
    text.push_str(label);
    text.push_str(":\n");
    text.push_str(&String::from_utf8_lossy(shown));
    if dropped > 0 {
        text.push_str(&format!("\n... {dropped} bytes truncated"));
    }
}
