//! Non-streaming response types and the assistant-turn mapping.

use serde::{Deserialize, Serialize};
use vox_domain::completion::AssistantTurn;
use vox_domain::message::{ContentBlock, Message, Role};

use super::chunks::WireUsage;

/// Non-streaming chat completion response.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// One or more choices; this adapter reads the first.
    pub choices: Vec<ResponseChoice>,
    /// Usage, present when requested.
    pub usage: Option<WireUsage>,
}

/// One choice of the response.
#[derive(Debug, Deserialize)]
pub struct ResponseChoice {
    /// The assistant message.
    pub message: ResponseMessage,
    /// Finish reason, for example `stop` or `tool_calls`.
    pub finish_reason: Option<String>,
}

/// The assistant message of a response.
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    /// Text content, when the model produced any.
    pub content: Option<String>,
    /// Tool calls the model requested.
    #[serde(default)]
    pub tool_calls: Vec<WireToolCall>,
}

/// A tool call requested by the model.
#[derive(Debug, Deserialize, Serialize)]
pub struct WireToolCall {
    /// Provider-issued call identifier.
    pub id: String,
    /// Call kind; always `function` for this milestone.
    #[serde(default = "default_kind")]
    pub kind: String,
    /// The invoked function.
    pub function: WireFunctionCall,
}

/// The invoked function of a tool call.
#[derive(Debug, Deserialize, Serialize)]
pub struct WireFunctionCall {
    /// Tool name.
    pub name: String,
    /// JSON-encoded arguments string.
    pub arguments: String,
}

/// Default call kind, used when the provider omits it.
fn default_kind() -> String {
    "function".to_string()
}

/// Parse a non-streaming response body into a domain assistant turn.
pub fn parse_completion(body: &str) -> Result<AssistantTurn, String> {
    let response: ChatResponse =
        serde_json::from_str(body).map_err(|error| format!("response decode failed: {error}"))?;
    let usage = response.usage.as_ref().map(WireUsage::to_domain);
    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| "response carried no choices".to_string())?;
    let mut content = Vec::new();
    if let Some(text) = choice.message.content {
        if !text.is_empty() {
            content.push(ContentBlock::Text { text });
        }
    }
    for call in choice.message.tool_calls {
        let arguments = serde_json::from_str(&call.function.arguments)
            .map_err(|error| format!("tool call arguments decode failed: {error}"))?;
        content.push(ContentBlock::ToolCall {
            id: call.id,
            name: call.function.name,
            arguments,
        });
    }
    Ok(AssistantTurn {
        message: Message {
            role: Role::Assistant,
            content,
        },
        finish_reason: choice.finish_reason,
        usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox_domain::completion::Usage;

    #[test]
    fn text_response_maps_to_a_turn() {
        let body = r#"{"choices":[{"message":{"content":"hello"},"finish_reason":"stop"}],"usage":{"prompt_tokens":3,"completion_tokens":4}}"#;
        let turn = parse_completion(body).expect("parse");
        assert_eq!(turn.finish_reason.as_deref(), Some("stop"));
        assert_eq!(
            turn.usage,
            Some(Usage {
                input_tokens: 3,
                output_tokens: 4
            })
        );
        assert_eq!(turn.message.text_content(), "hello");
    }

    #[test]
    fn tool_calls_map_to_content_blocks() {
        let body = r#"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"c1","function":{"name":"read","arguments":"{\"path\":\"a.txt\"}"}}]},"finish_reason":"tool_calls"}]}"#;
        let turn = parse_completion(body).expect("parse");
        assert_eq!(turn.finish_reason.as_deref(), Some("tool_calls"));
        match &turn.message.content[0] {
            ContentBlock::ToolCall {
                id,
                name,
                arguments,
            } => {
                assert_eq!(id, "c1");
                assert_eq!(name, "read");
                assert_eq!(arguments["path"], "a.txt");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn malformed_body_fails_loudly() {
        assert!(parse_completion("not json").is_err());
        assert!(parse_completion(r#"{"choices":[]}"#).is_err());
    }
}
