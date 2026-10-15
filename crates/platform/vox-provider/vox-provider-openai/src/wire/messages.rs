//! Outgoing message mapping, including tool calls and results.

use serde::Serialize;
use vox_domain::message::{ContentBlock, Message, Role};

use super::response::WireToolCall;

/// Outgoing message.
#[derive(Debug, Serialize)]
pub struct WireMessage {
    /// Wire role name.
    pub role: String,
    /// Text content, when the message carries any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Assistant tool calls, when the message carries any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<WireToolCall>>,
    /// Call id this `tool` role message answers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
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
/// One domain message can yield several wire messages: assistant tool
/// calls ride the `tool_calls` array, and each tool result becomes its
/// own `tool` role message keyed by call id. Text content is
/// concatenated across text blocks and rides its own message whenever
/// the message does not exist only to carry tool results.
pub fn to_wire_messages(messages: &[Message]) -> Result<Vec<WireMessage>, String> {
    let mut wire = Vec::new();
    for message in messages {
        let mut text = String::new();
        let mut tool_calls = Vec::new();
        let mut results = Vec::new();
        for block in &message.content {
            match block {
                ContentBlock::Text { text: chunk } => text.push_str(chunk),
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                } => tool_calls.push(WireToolCall {
                    id: id.clone(),
                    kind: "function".to_string(),
                    function: super::response::WireFunctionCall {
                        name: name.clone(),
                        arguments: arguments.to_string(),
                    },
                }),
                ContentBlock::ToolResult { call_id, content } => {
                    results.push((call_id.clone(), content.clone()));
                }
            }
        }
        // Text rides its own message unless the domain message exists
        // only to carry tool results: their content is the result
        // payload, and a calls-only turn with no text emits the call
        // array alone. The explicit `&&` pins the precedence: no
        // parenthesised surprise when this is read later.
        if !text.is_empty() && results.is_empty() {
            wire.push(WireMessage {
                role: role_name(message.role).to_string(),
                content: Some(text),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        if !tool_calls.is_empty() {
            wire.push(WireMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(tool_calls),
                tool_call_id: None,
            });
        }
        for (call_id, content) in results {
            wire.push(WireMessage {
                role: "tool".to_string(),
                content: Some(content),
                tool_calls: None,
                tool_call_id: Some(call_id),
            });
        }
    }
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_messages_map_to_wire_form() {
        let messages = vec![Message::system("rules"), Message::user("hi")];
        let wire = to_wire_messages(&messages).expect("map");
        assert_eq!(wire[0].role, "system");
        assert_eq!(wire[1].content.as_deref(), Some("hi"));
    }

    #[test]
    fn tool_calls_and_results_map_to_their_wire_shapes() {
        let call = Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: "c1".into(),
                name: "read".into(),
                arguments: serde_json::json!({"path": "a.txt"}),
            }],
        };
        let result = Message {
            role: Role::Tool,
            content: vec![ContentBlock::ToolResult {
                call_id: "c1".into(),
                content: "done".into(),
            }],
        };
        let wire = to_wire_messages(&[call, result]).expect("map");
        let assistant = wire.iter().find(|m| m.tool_calls.is_some()).expect("call");
        let call = &assistant.tool_calls.as_ref().expect("calls")[0];
        assert_eq!(call.function.name, "read");
        assert!(call.function.arguments.contains("a.txt"));
        let tool = wire.iter().find(|m| m.role == "tool").expect("result");
        assert_eq!(tool.tool_call_id.as_deref(), Some("c1"));
        assert_eq!(tool.content.as_deref(), Some("done"));
    }
}
