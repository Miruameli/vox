//! The agent run: a bounded, durable tool-use loop.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use vox_domain::completion::CompletionRequest;
use vox_domain::message::Message;
use vox_domain::provider::ModelProvider;
use vox_engine_tools::ToolHarness;
use vox_session::{EventKind, SessionWriter};
use vox_shared::VoxResult;

use crate::bounds::{FinishReason, Limits};
use crate::events::LoopEvent;
use crate::resume::{messages_from_events, tool_calls_of, tool_result_message};

/// An agent run: one prompt, bounded turns, durable events.
pub struct Agent<P: ModelProvider> {
    provider: P,
    harness: ToolHarness,
    session: SessionWriter,
    session_path: PathBuf,
    model: String,
    limits: Limits,
    cancelled: Arc<AtomicBool>,
    messages: Vec<Message>,
}

impl<P: ModelProvider> Agent<P> {
    /// Assemble an agent that appends its events to `session_path`.
    pub fn new(
        provider: P,
        harness: ToolHarness,
        session_path: &Path,
        model: impl Into<String>,
        limits: Limits,
    ) -> VoxResult<Self> {
        let session = SessionWriter::open(session_path)?;
        Ok(Self {
            provider,
            harness,
            session,
            session_path: session_path.to_path_buf(),
            model: model.into(),
            limits,
            cancelled: Arc::new(AtomicBool::new(false)),
            messages: Vec::new(),
        })
    }

    /// Seed the conversation once, before the first `run` call.
    ///
    /// A resume rebuilds the stored conversation and hands it over
    /// here; every later `run` continues from the agent's own
    /// growing conversation instead.
    pub fn with_prior(mut self, prior: Vec<Message>) -> Self {
        self.messages = prior;
        self
    }

    /// Shared cancellation flag; set it to stop between actions.
    pub fn cancel_token(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    /// Rebuild the conversation of this agent's session, for resume.
    pub fn resume_messages(&self) -> VoxResult<Vec<Message>> {
        let text = std::fs::read_to_string(&self.session_path)?;
        Ok(messages_from_events(&vox_session::replay(&text)?))
    }

    /// Run one prompt, continuing the agent's conversation.
    ///
    /// Every message, tool call, and tool result is appended to the
    /// session before the next provider request, so a crash resumes
    /// from the file with nothing lost. Each call extends the same
    /// conversation: a second `run` in one process sees everything
    /// the first one produced.
    pub async fn run(
        &mut self,
        prompt: &str,
        sink: &mut dyn FnMut(LoopEvent),
    ) -> VoxResult<FinishReason> {
        let user = Message::user(prompt);
        self.session.append(EventKind::Message {
            message: user.clone(),
        })?;
        self.messages.push(user);
        let mut consecutive_tool_errors = 0usize;

        for turn in 1..=self.limits.max_turns {
            if self.cancelled.load(Ordering::SeqCst) {
                return self.finish(sink, FinishReason::Cancelled);
            }
            sink(LoopEvent::TurnStarted { turn });
            let request = CompletionRequest::with_tools(
                self.model.clone(),
                self.messages.clone(),
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
            self.messages.push(assistant.message);
            if !wants_tools || tool_calls.is_empty() {
                return self.finish(sink, FinishReason::ModelStopped);
            }

            for call in tool_calls {
                if self.cancelled.load(Ordering::SeqCst) {
                    return self.finish(sink, FinishReason::Cancelled);
                }
                sink(LoopEvent::ToolStarted {
                    name: call.name.clone(),
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
                });
                consecutive_tool_errors = if result.is_error {
                    consecutive_tool_errors + 1
                } else {
                    0
                };
                if consecutive_tool_errors >= self.limits.max_consecutive_tool_errors {
                    return self.finish(sink, FinishReason::ToolErrorLimit);
                }
                self.messages.push(tool_result_message(&result));
            }
        }
        self.finish(sink, FinishReason::TurnLimit)
    }

    /// Emit the terminal event and return the reason.
    fn finish(
        &self,
        sink: &mut dyn FnMut(LoopEvent),
        reason: FinishReason,
    ) -> VoxResult<FinishReason> {
        sink(LoopEvent::Finished { reason });
        Ok(reason)
    }
}
