//! The tool-use loop: one prompt, bounded turns, durable events.
//!
//! [`Agent::continue_conversation`] is the entry point the agent's
//! callers use. The loop sends the conversation plus the tool
//! definitions to the provider, executes every requested tool through
//! the harness, and appends every message, tool call, and tool result
//! to the session before the next request. The bounds and the
//! cancellation checks of [`crate::contract::Limits`] live here; the
//! assembly of an agent lives in [`super::agent`].

use std::sync::atomic::Ordering;

use vox_domain::completion::CompletionRequest;
use vox_domain::message::Message;
use vox_domain::provider::ModelProvider;
use vox_session::EventKind;
use vox_shared::VoxResult;

use crate::agent::Agent;
use crate::contract::{FinishReason, LoopEvent};
use crate::resume::{tool_calls_of, tool_result_message};

impl<P: ModelProvider> Agent<P> {
    /// Continue a conversation with one more prompt.
    ///
    /// `history` carries the turns that already happened and stays
    /// owned by the caller, so a surface that lives for many prompts
    /// (the TUI) keeps the whole session in context. The new user
    /// message is appended to `history` and to the session, then the
    /// loop runs to its end: every assistant message, tool call, and
    /// tool result is appended to both before the next provider
    /// request. `history` is written back even when the loop fails, so
    /// a failed turn can be retried without replaying the session.
    pub async fn continue_conversation(
        &mut self,
        history: &mut Vec<Message>,
        prompt: &str,
        sink: &mut (dyn FnMut(LoopEvent) + Send),
    ) -> VoxResult<FinishReason> {
        let user = Message::user(prompt);
        self.session.append(EventKind::Message {
            message: user.clone(),
        })?;
        let mut messages = std::mem::take(history);
        messages.push(user);
        let outcome = self.drive(&mut messages, sink).await;
        *history = messages;
        outcome
    }

    /// Drive the tool-use loop over a seeded conversation.
    ///
    /// The bounds, the durability rule (append before the next
    /// request), and the cancellation checks of
    /// [`Self::continue_conversation`] all live here; `messages` is
    /// extended in place so a failed loop still leaves its partial
    /// conversation with the caller.
    async fn drive(
        &mut self,
        messages: &mut Vec<Message>,
        sink: &mut (dyn FnMut(LoopEvent) + Send),
    ) -> VoxResult<FinishReason> {
        let mut consecutive_tool_errors = 0usize;

        for turn in 1..=self.limits.max_turns {
            if self.cancelled.load(Ordering::SeqCst) {
                return self.finish(sink, FinishReason::Cancelled);
            }
            sink(LoopEvent::TurnStarted { turn });
            let request = CompletionRequest::with_tools(
                self.model.clone(),
                messages.clone(),
                vox_engine_tools::definitions(),
            );
            let assistant = self.provider.complete(&request).await?;
            self.session.append(EventKind::Message {
                message: assistant.message.clone(),
            })?;
            sink(LoopEvent::Text {
                text: assistant.message.text_content(),
            });

            let tool_calls = tool_calls_of(&assistant.message);
            let wants_tools = assistant.finish_reason.as_deref() == Some("tool_calls");
            messages.push(assistant.message);
            if !wants_tools || tool_calls.is_empty() {
                return self.finish(sink, FinishReason::ModelStopped);
            }

            for call in tool_calls {
                if self.cancelled.load(Ordering::SeqCst) {
                    return self.finish(sink, FinishReason::Cancelled);
                }
                sink(LoopEvent::ToolStarted {
                    name: call.name.clone(),
                    detail: crate::contract::arguments_preview(&call.arguments),
                });
                self.session
                    .append(EventKind::ToolCall { call: call.clone() })?;
                let result = self.harness.call(&call)?;
                self.session.append(EventKind::ToolResult {
                    result: result.clone(),
                })?;
                sink(LoopEvent::ToolFinished {
                    name: call.name.clone(),
                    is_error: result.is_error,
                    detail: crate::contract::preview(&result.content),
                });
                consecutive_tool_errors = if result.is_error {
                    consecutive_tool_errors + 1
                } else {
                    0
                };
                if consecutive_tool_errors >= self.limits.max_consecutive_tool_errors {
                    return self.finish(sink, FinishReason::ToolErrorLimit);
                }
                messages.push(tool_result_message(&result));
            }
        }
        self.finish(sink, FinishReason::TurnLimit)
    }

    /// Emit the terminal event and return the reason.
    fn finish(
        &self,
        sink: &mut (dyn FnMut(LoopEvent) + Send),
        reason: FinishReason,
    ) -> VoxResult<FinishReason> {
        sink(LoopEvent::Finished { reason });
        Ok(reason)
    }
}
