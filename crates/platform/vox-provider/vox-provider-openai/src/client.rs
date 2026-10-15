//! OpenAI-compatible streaming client.

use vox_domain::completion::{CompletionRequest, StreamEvent};
use vox_domain::provider::ModelProvider;
use vox_shared::{VoxError, VoxResult};

use crate::consume::consume_stream;
use crate::wire::{to_wire_messages, ChatRequest, StreamOptions};

/// Client for the OpenAI-compatible chat completions API.
#[derive(Debug, Clone)]
pub struct OpenAiClient {
    /// API root ending in `/v1`.
    base_url: String,
    /// Bearer token sent on every request.
    api_key: String,
    /// Shared HTTP client.
    http: reqwest::Client,
}

impl OpenAiClient {
    /// Create a client; `base_url` is the API root ending in `/v1`.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Build the wire request for a domain request.
    fn wire_request(&self, request: &CompletionRequest) -> VoxResult<ChatRequest> {
        let messages =
            to_wire_messages(&request.messages).map_err(|detail| VoxError::Provider {
                detail: format!("message mapping failed: {detail}"),
            })?;
        Ok(ChatRequest {
            model: request.model.clone(),
            messages,
            stream: request.stream,
            stream_options: StreamOptions {
                include_usage: true,
            },
            max_tokens: request.max_tokens,
        })
    }
}

impl ModelProvider for OpenAiClient {
    fn id(&self) -> &str {
        "openai"
    }

    async fn stream_completion(
        &self,
        request: &CompletionRequest,
        on_event: &mut (dyn FnMut(StreamEvent) + Send),
    ) -> VoxResult<()> {
        let wire = self.wire_request(request)?;
        let response = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&wire)
            .send()
            .await
            .map_err(|error| VoxError::Http {
                detail: error.to_string(),
            })?;
        let status = response.status();
        if !status.is_success() {
            let detail = format!("chat completions returned HTTP {}", status.as_u16());
            tracing::debug!(%detail, "provider rejected request");
            return Err(VoxError::Provider { detail });
        }
        consume_stream(response, on_event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox_domain::message::Message;

    #[test]
    fn wire_request_enables_usage_and_streaming() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "key");
        let request =
            CompletionRequest::streaming("m", vec![Message::user("hi"), Message::system("rules")]);
        let wire = client.wire_request(&request).expect("wire");
        assert!(wire.stream);
        assert!(wire.stream_options.include_usage);
        assert_eq!(wire.messages.len(), 2);
        assert_eq!(wire.messages[1].role, "system");
    }

    #[test]
    fn client_reports_provider_id() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "key");
        assert_eq!(client.id(), "openai");
    }
}
