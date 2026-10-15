//! Streaming chunk types and usage conversion.

use serde::Deserialize;
use vox_domain::completion::Usage;

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

impl WireUsage {
    /// Convert to domain usage.
    pub fn to_domain(&self) -> Usage {
        Usage {
            input_tokens: self.prompt_tokens,
            output_tokens: self.completion_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_parses_delta_and_usage() {
        let json = r#"{"model":"m","choices":[{"delta":{"content":"hi"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":2}}"#;
        let chunk: ChatChunk = serde_json::from_str(json).expect("parse");
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("hi"));
        assert_eq!(chunk.choices[0].finish_reason.as_deref(), Some("stop"));
        let usage = chunk.usage.expect("usage");
        assert_eq!(usage.to_domain().input_tokens, 1);
    }
}
