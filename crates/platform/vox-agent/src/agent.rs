//! The agent run: a bounded, durable tool-use loop.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use vox_domain::message::Message;
use vox_domain::provider::ModelProvider;
use vox_engine_tools::ToolHarness;
use vox_session::SessionWriter;
use vox_shared::VoxResult;

use crate::contract::{FinishReason, Limits, LoopEvent};
use crate::resume::messages_from_events;

/// An agent run: one prompt, bounded turns, durable events.
///
/// The fields are visible to the crate, not the public API: the loop
/// in [`crate::run_loop`] drives them, and callers outside this
/// crate can only reach the constructor, the cancellation token, and
/// the run.
pub struct Agent<P: ModelProvider> {
    pub(crate) provider: P,
    pub(crate) harness: ToolHarness,
    pub(crate) session: SessionWriter,
    pub(crate) session_path: PathBuf,
    pub(crate) model: String,
    pub(crate) limits: Limits,
    pub(crate) cancelled: Arc<AtomicBool>,
    /// The agent's own conversation: everything earlier `run` calls
    /// produced, seeded once by [`Self::with_prior`]. Surfaces that
    /// own their history (the TUI) use
    /// [`Self::continue_conversation`] instead and never touch it.
    pub(crate) messages: Vec<Message>,
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
    /// the first one produced. Surfaces that own their history (the
    /// TUI) use [`Self::continue_conversation`] instead.
    pub async fn run(
        &mut self,
        prompt: &str,
        sink: &mut (dyn FnMut(LoopEvent) + Send),
    ) -> VoxResult<FinishReason> {
        // The conversation is taken out for the run and written back
        // afterwards, so a failed turn keeps its partial conversation
        // for the next call instead of retrying from scratch.
        let mut history = std::mem::take(&mut self.messages);
        let reason = self.continue_conversation(&mut history, prompt, sink).await;
        self.messages = history;
        reason
    }
}
