//! Completion request and stream event types.

use serde::{Deserialize, Serialize};

use crate::message::Message;

/// A completion request sent to a provider.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Model identifier to invoke.
    pub model: String,
    /// Conversation messages in chronological order.
    pub messages: Vec<Message>,
    /// Maximum tokens to generate, when the provider supports the knob.
    pub max_tokens: Option<u32>,
    /// Whether the response is streamed incrementally.
    pub stream: bool,
}

impl CompletionRequest {
    /// Build a streaming request with no token cap.
    pub fn streaming(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens: None,
            stream: true,
        }
    }
}

/// Token accounting for one completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens consumed by the prompt.
    pub input_tokens: u32,
    /// Tokens produced by the model.
    pub output_tokens: u32,
}

/// Events emitted while streaming a completion.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    /// The stream opened; carries the resolved model name.
    Started {
        /// Model name reported by the provider.
        model: String,
    },
    /// Incremental text output.
    Delta {
        /// Text fragment.
        text: String,
    },
    /// The model finished the turn.
    Completed {
        /// Provider finish reason, when reported.
        finish_reason: Option<String>,
        /// Token usage, when reported.
        usage: Option<Usage>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Role;

    #[test]
    fn streaming_request_defaults_are_stable() {
        let request = CompletionRequest::streaming("gpt-4o-mini", vec![Message::user("hi")]);
        assert_eq!(request.model, "gpt-4o-mini");
        assert!(request.stream);
        assert!(request.max_tokens.is_none());
        assert_eq!(request.messages[0].role, Role::User);
    }

    #[test]
    fn usage_round_trips_through_json() {
        let usage = Usage {
            input_tokens: 7,
            output_tokens: 3,
        };
        let json = serde_json::to_string(&usage).expect("serialize");
        assert_eq!(json, "{\"input_tokens\":7,\"output_tokens\":3}");
    }
}
