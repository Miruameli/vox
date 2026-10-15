//! Provider abstraction implemented by infrastructure crates.

use std::future::Future;

use vox_shared::{VoxError, VoxResult};

use crate::completion::{AssistantTurn, CompletionRequest, StreamEvent};

/// A model provider able to stream completions.
///
/// Implementations live in `platform/` crates and translate between this
/// domain contract and their wire protocol. The methods return futures so
/// auto-trait bounds can be expressed (see `Send` below).
pub trait ModelProvider: Send + Sync {
    /// Stable identifier of the provider, for example `openai`.
    fn id(&self) -> &str;

    /// Stream one completion, invoking `on_event` for every event.
    ///
    /// On success the final event is always [`StreamEvent::Completed`].
    /// On failure no `Completed` event is emitted and the error carries only
    /// a generic user-facing message; diagnostics go to the log.
    fn stream_completion(
        &self,
        request: &CompletionRequest,
        on_event: &mut (dyn FnMut(StreamEvent) + Send),
    ) -> impl Future<Output = VoxResult<()>> + Send;

    /// Complete one non-streaming turn with tools, for the agent loop.
    ///
    /// The response carries the full assistant message including tool
    /// calls; the loop dispatches them and sends the results back.
    fn complete(
        &self,
        request: &CompletionRequest,
    ) -> impl Future<Output = VoxResult<AssistantTurn>> + Send;
}

/// Error returned when a message cannot be mapped to the provider's wire
/// format (for example tool calls on a text-only adapter).
pub fn unsupported_content(detail: impl Into<String>) -> VoxError {
    VoxError::Provider {
        detail: detail.into(),
    }
}
