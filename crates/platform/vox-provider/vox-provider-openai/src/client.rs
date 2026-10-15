//! OpenAI-compatible chat completions client.

use vox_domain::completion::{AssistantTurn, CompletionRequest, StreamEvent};
use vox_domain::provider::ModelProvider;
use vox_shared::{VoxError, VoxResult};

use crate::consume::consume_stream;
use crate::wire::{build, parse_completion, ChatRequest};

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

    /// POST one wire request, mapping transport and status failures.
    async fn post(&self, wire: &ChatRequest) -> VoxResult<reqwest::Response> {
        let response = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(wire)
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
        Ok(response)
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
        let wire = build(request).map_err(|detail| VoxError::Provider {
            detail: format!("message mapping failed: {detail}"),
        })?;
        let response = self.post(&wire).await?;
        consume_stream(response, on_event).await
    }

    async fn complete(&self, request: &CompletionRequest) -> VoxResult<AssistantTurn> {
        let wire = build(request).map_err(|detail| VoxError::Provider {
            detail: format!("message mapping failed: {detail}"),
        })?;
        let response = self.post(&wire).await?;
        let body = response.text().await.map_err(|error| VoxError::Http {
            detail: error.to_string(),
        })?;
        parse_completion(&body).map_err(|detail| VoxError::Stream { detail })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox_domain::message::Message;

    #[test]
    fn client_reports_provider_id() {
        assert_eq!(OpenAiClient::new("http://x/v1", "k").id(), "openai");
    }

    #[test]
    fn client_is_a_model_provider() {
        fn assert_provider<T: ModelProvider>() {}
        assert_provider::<OpenAiClient>();
    }

    #[test]
    fn requests_are_built_from_domain_messages() {
        let client = OpenAiClient::new("http://x/v1", "k");
        let request = CompletionRequest::streaming("m", vec![Message::user("hi")]);
        let wire = build(&request).expect("build");
        assert_eq!(wire.messages.len(), 1);
        assert_eq!(wire.model, "m");
        // The client itself stays a thin transport: no message assembly.
        assert!(client.base_url.ends_with("/v1"));
    }
}
