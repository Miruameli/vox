//! Tool definition, call, and result types.

use serde::{Deserialize, Serialize};

/// Description of a tool the model may call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name as referenced by the model.
    pub name: String,
    /// Human- and model-readable description.
    pub description: String,
    /// JSON Schema describing the arguments object.
    pub parameters: serde_json::Value,
}

/// A tool invocation requested by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Provider-issued call identifier.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Tool arguments as JSON.
    pub arguments: serde_json::Value,
}

/// The outcome of executing a tool call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Identifier of the answered call.
    pub call_id: String,
    /// Output handed back to the model.
    pub content: String,
    /// Whether the tool failed.
    pub is_error: bool,
}

impl ToolResult {
    /// Build a successful result.
    pub fn success(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Build a failed result.
    pub fn failure(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            content: content.into(),
            is_error: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_constructors_set_error_flag() {
        let ok = ToolResult::success("c1", "done");
        let bad = ToolResult::failure("c2", "boom");
        assert!(!ok.is_error);
        assert!(bad.is_error);
    }
}
