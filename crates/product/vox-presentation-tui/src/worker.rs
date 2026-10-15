//! The agent bridge: one worker task owns the agent.
//!
//! Prompts travel to the worker over a channel and events travel back.
//! Keeping the agent on its own task is what makes the terminal
//! responsive: the render loop never waits on a provider, a turn that
//! takes minutes cannot freeze the keys, and the interrupt key stays
//! live while the model is thinking.
//!
//! The worker owns the conversation, so a surface that submits prompt
//! after prompt keeps the whole session in context
//! ([`Agent::continue_conversation`]) while the durable session file
//! stays the authority on every event.

use std::io::Error as IoError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc;
use vox_agent::Agent;
use vox_domain::message::Message;
use vox_domain::provider::ModelProvider;
use vox_shared::{VoxError, VoxResult};

use crate::app::AppMsg;

/// The bridge between the event loop and the agent task.
pub struct Worker {
    prompts: mpsc::UnboundedSender<String>,
    cancel: Arc<AtomicBool>,
}

impl Worker {
    /// Spawn the bridge over `agent`, sending events to `events`.
    ///
    /// `cancel` is the agent's own cancellation flag: the event loop
    /// sets it and the loop stops at the next safe point, which is the
    /// contract the agent already documents. `prior` is the
    /// conversation a resumed session starts from.
    pub fn spawn<P: ModelProvider + 'static>(
        agent: Agent<P>,
        events: mpsc::UnboundedSender<AppMsg>,
        cancel: Arc<AtomicBool>,
        prior: Vec<Message>,
    ) -> Self {
        // Prompts are named as `String` because the loop dereferences
        // them to `&str`; naming the type keeps the channel from being
        // inferred as one carrying `str` itself.
        let (prompts, mut inbox) = mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            let mut agent = agent;
            let mut conversation: Vec<Message> = prior;
            while let Some(prompt) = inbox.recv().await {
                let outcome = agent
                    .continue_conversation(&mut conversation, &prompt, &mut |event| {
                        let _ = events.send(AppMsg::Agent(event));
                    })
                    .await;
                // The loop already reports how a run ended through
                // `LoopEvent::Finished`, so only a failure needs a
                // message of its own - the loop has no event for one.
                if let Err(error) = outcome {
                    if events.send(AppMsg::Failed(error)).is_err() {
                        break;
                    }
                }
            }
        });
        Self { prompts, cancel }
    }

    /// Send one prompt to the worker.
    pub fn submit(&self, prompt: String) -> VoxResult<()> {
        self.prompts
            .send(prompt)
            .map_err(|_| VoxError::Io(IoError::other("the agent worker stopped")))
    }

    /// Stop the run at the agent's next safe point.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}
