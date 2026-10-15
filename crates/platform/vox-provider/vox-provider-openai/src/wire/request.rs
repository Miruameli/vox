//! Outgoing request types: the chat completion request and tools.

use serde::Serialize;

use vox_domain::completion::CompletionRequest;

use super::messages::{to_wire_messages, WireMessage};

/// Outgoing chat completion request.
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    /// Model identifier.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<WireMessage>,
    /// Whether the response streams incrementally.
    pub stream: bool,
    /// Streaming options; usage is always requested.
    pub stream_options: StreamOptions,
    /// Optional output token cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Tools offered to the model, when any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<WireTool>>,
}

/// Streaming options requested from the provider.
#[derive(Debug, Serialize)]
pub struct StreamOptions {
    /// Ask the provider to include a usage chunk before `[DONE]`.
    pub include_usage: bool,
}

/// A function tool offered to the model.
#[derive(Debug, Serialize)]
pub struct WireTool {
    /// Always `function` for this milestone.
    #[serde(rename = "type")]
    pub kind: String,
    /// The function definition.
    pub function: WireFunction,
}

/// The function definition inside a tool.
#[derive(Debug, Serialize)]
pub struct WireFunction {
    /// Tool name as referenced by the model.
    pub name: String,
    /// Human- and model-readable description.
    pub description: String,
    /// JSON Schema describing the arguments object.
    pub parameters: serde_json::Value,
}

impl WireTool {
    /// Build a wire tool from a domain tool definition.
    pub fn from_definition(definition: &vox_domain::tool::ToolDefinition) -> Self {
        Self {
            kind: "function".to_string(),
            function: WireFunction {
                name: definition.name.clone(),
                description: definition.description.clone(),
                parameters: definition.parameters.clone(),
            },
        }
    }
}

/// Build the wire request for a domain request.
pub fn build(request: &CompletionRequest) -> Result<ChatRequest, String> {
    let messages = to_wire_messages(&request.messages)?;
    let tools = request
        .tools
        .as_ref()
        .map(|definitions| definitions.iter().map(WireTool::from_definition).collect());
    Ok(ChatRequest {
        model: request.model.clone(),
        messages,
        stream: request.stream,
        stream_options: StreamOptions {
            include_usage: true,
        },
        max_tokens: request.max_tokens,
        tools,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox_domain::message::Message;
    use vox_domain::tool::ToolDefinition;

    #[test]
    fn request_carries_tools_and_usage_options() {
        let tool = ToolDefinition {
            name: "read".to_string(),
            description: "read a file".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        };
        let request = CompletionRequest::with_tools("m", vec![Message::user("hi")], vec![tool]);
        let wire = build(&request).expect("build");
        assert!(wire.stream_options.include_usage);
        assert!(!wire.stream);
        let tools = wire.tools.expect("tools");
        assert_eq!(tools[0].function.name, "read");
    }

    #[test]
    fn streaming_request_omits_tools() {
        let wire = build(&CompletionRequest::streaming(
            "m",
            vec![Message::user("hi")],
        ))
        .expect("build");
        assert!(wire.tools.is_none());
        assert!(wire.stream);
    }
}
