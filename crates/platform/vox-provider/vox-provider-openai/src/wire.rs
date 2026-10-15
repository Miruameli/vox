//! Wire types for the OpenAI-compatible chat completions API.

use serde::{Deserialize, Serialize};

use vox_domain::message::{ContentBlock, Message, Role};

/// Sentinel payload terminating a chat completion stream.
pub const DONE_SENTINEL: &str = "[DONE]";

/// Outgoing chat completion request.
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    /// Model identifier.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<WireMessage>,
    /// Always true: the adapter streams.
    pub stream: bool,
    /// Request usage in the final stream chunk.
    pub stream_options: StreamOptions,
    /// Optional output token cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// Streaming options requested from the provider.
#[derive(Debug, Serialize)]
pub struct StreamOptions {
    /// Ask the provider to include a usage chunk before `[DONE]`.
    pub include_usage: bool,
}

/// Outgoing message; this adapter is text-only for the first milestone.
#[derive(Debug, Serialize)]
pub struct WireMessage {
    /// Wire role name.
    pub role: String,
    /// Text content.
    pub content: String,
}

/// Incoming stream chunk.
#[derive(Debug, Deserialize)]
pub struct ChatChunk {
    /// Model name reported by the provider.
    pub model: Option<String>,
    /// One or more choices; streaming yields exactly one.
    pub choices: Vec<ChunkChoice>,
    /// Usage, present on the final chunk when requested.
    pub usage: Option<WireUsage>,
}

/// One choice inside a stream chunk.
#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    /// Incremental content.
    pub delta: ChunkDelta,
    /// Finish reason, present on the last content chunk.
    pub finish_reason: Option<String>,
}

/// Incremental content of a choice.
#[derive(Debug, Deserialize)]
pub struct ChunkDelta {
    /// Text fragment.
    pub content: Option<String>,
}

/// Token usage reported by the provider.
#[derive(Debug, Deserialize)]
pub struct WireUsage {
    /// Prompt tokens.
    pub prompt_tokens: u32,
    /// Completion tokens.
    pub completion_tokens: u32,
}

/// Convert a domain role to its wire name.
pub fn role_name(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

/// Map domain messages to wire messages.
///
/// Only text content is supported in this milestone; tool calls and tool
/// results are rejected so silent truncation cannot occur.
pub fn to_wire_messages(messages: &[Message]) -> Result<Vec<WireMessage>, String> {
    messages
        .iter()
        .map(|message| {
            let mut text = String::new();
            for block in &message.content {
                match block {
                    ContentBlock::Text { text: chunk } => text.push_str(chunk),
                    ContentBlock::ToolCall { .. } | ContentBlock::ToolResult { .. } => {
                        return Err(format!(
                            "role {} carries tool content unsupported by this adapter",
                            role_name(message.role)
                        ));
                    }
                }
            }
            Ok(WireMessage {
                role: role_name(message.role).to_string(),
                content: text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox_domain::message::Message;

    #[test]
    fn text_messages_map_to_wire_form() {
        let messages = vec![Message::system("rules"), Message::user("hi")];
        let wire = to_wire_messages(&messages).expect("map");
        assert_eq!(wire[0].role, "system");
        assert_eq!(wire[1].content, "hi");
    }

    #[test]
    fn tool_content_is_rejected() {
        let message = Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: "c1".into(),
                name: "read".into(),
                arguments: serde_json::json!({}),
            }],
        };
        let error = to_wire_messages(&[message]).expect_err("reject");
        assert!(error.contains("tool content"));
    }

    #[test]
    fn chunk_parses_delta_and_usage() {
        let json = r#"{"model":"m","choices":[{"delta":{"content":"hi"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":2}}"#;
        let chunk: ChatChunk = serde_json::from_str(json).expect("parse");
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("hi"));
        assert_eq!(chunk.choices[0].finish_reason.as_deref(), Some("stop"));
        assert_eq!(chunk.usage.expect("usage").prompt_tokens, 1);
    }
}
