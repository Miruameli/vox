//! Conversation domain types.

use serde::{Deserialize, Serialize};

/// Role of a message author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System instructions.
    System,
    /// End-user input.
    User,
    /// Model output.
    Assistant,
    /// Result of a tool invocation.
    Tool,
}

/// One part of a message's content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content.
    Text {
        /// The text itself.
        text: String,
    },
    /// A tool invocation requested by the model.
    ToolCall {
        /// Provider-issued call identifier.
        id: String,
        /// Tool name.
        name: String,
        /// Tool arguments as JSON.
        arguments: serde_json::Value,
    },
    /// The result of a tool invocation.
    ToolResult {
        /// Identifier of the call this result answers.
        call_id: String,
        /// Tool output.
        content: String,
    },
}

/// A single conversation message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Author of the message.
    pub role: Role,
    /// Ordered content blocks.
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Build a text-only message.
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    /// Build a user text message.
    pub fn user(text: impl Into<String>) -> Self {
        Self::text(Role::User, text)
    }

    /// Build a system text message.
    pub fn system(text: impl Into<String>) -> Self {
        Self::text(Role::System, text)
    }

    /// Concatenated text of all text blocks.
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_message_round_trips_through_json() {
        let message = Message::user("list files");
        let json = serde_json::to_string(&message).expect("serialize");
        let parsed: Message = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, message);
        assert_eq!(parsed.text_content(), "list files");
    }

    #[test]
    fn role_serializes_lowercase() {
        let json = serde_json::to_string(&Role::Assistant).expect("serialize");
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn tool_blocks_are_tagged() {
        let block = ContentBlock::ToolCall {
            id: "call_1".into(),
            name: "read".into(),
            arguments: serde_json::json!({"path": "a.rs"}),
        };
        let json = serde_json::to_string(&block).expect("serialize");
        assert!(json.contains("\"type\":\"tool_call\""));
    }
}
