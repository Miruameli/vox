//! Stream consumption: turns an SSE body into domain stream events.

use futures_util::StreamExt;
use vox_domain::completion::{StreamEvent, Usage};
use vox_shared::{VoxError, VoxResult};

use crate::stream::SseDecoder;
use crate::wire::{ChatChunk, DONE_SENTINEL};

/// Read the SSE body and emit domain stream events.
///
/// Exactly one [`StreamEvent::Completed`] is emitted on success, whether the
/// stream ends with the `[DONE]` sentinel, a finish reason, or exhaustion.
pub(crate) async fn consume_stream(
    response: reqwest::Response,
    on_event: &mut (dyn FnMut(StreamEvent) + Send),
) -> VoxResult<()> {
    let mut decoder = SseDecoder::new();
    let mut body = response.bytes_stream();
    let mut completed = false;
    let mut usage: Option<Usage> = None;
    while let Some(chunk) = body.next().await {
        let bytes = chunk.map_err(|error| VoxError::Http {
            detail: error.to_string(),
        })?;
        for payload in decoder.push(&bytes)? {
            if payload == DONE_SENTINEL {
                finish(&mut completed, None, usage, on_event);
                return Ok(());
            }
            let parsed: ChatChunk =
                serde_json::from_str(&payload).map_err(|error| VoxError::Stream {
                    detail: format!("chunk decode failed: {error}"),
                })?;
            if parsed.usage.is_some() {
                usage = parsed.usage.as_ref().map(convert_usage);
            }
            for choice in parsed.choices {
                if let Some(text) = choice.delta.content {
                    if !text.is_empty() {
                        on_event(StreamEvent::Delta { text });
                    }
                }
                if choice.finish_reason.is_some() {
                    finish(
                        &mut completed,
                        choice.finish_reason.clone(),
                        usage,
                        on_event,
                    );
                }
            }
        }
    }
    if !completed {
        finish(&mut completed, None, usage, on_event);
    }
    Ok(())
}

/// Convert wire usage to domain usage.
fn convert_usage(usage: &crate::wire::WireUsage) -> Usage {
    Usage {
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
    }
}

/// Emit the single completion event, logging duplicates.
fn finish(
    completed: &mut bool,
    finish_reason: Option<String>,
    usage: Option<Usage>,
    on_event: &mut (dyn FnMut(StreamEvent) + Send),
) {
    if *completed {
        tracing::debug!("duplicate completion event dropped");
        return;
    }
    *completed = true;
    on_event(StreamEvent::Completed {
        finish_reason,
        usage,
    });
}
