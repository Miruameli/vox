//! A provider that replays scripted turns and records what it was sent.
//!
//! The bridge is driven by a provider whose every response is known, so
//! a run is deterministic and the test can assert on both what the view
//! received and what the provider was asked for.

use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};

use serde_json::json;
use tempfile::TempDir;
use vox_agent::{Agent, Limits};
use vox_domain::completion::{AssistantTurn, CompletionRequest, StreamEvent};
use vox_domain::message::{ContentBlock, Message, Role};
use vox_domain::provider::ModelProvider;
use vox_engine_tools::ToolHarness;
use vox_shared::VoxResult;

/// A provider replaying scripted turns, oldest first.
pub struct Scripted {
    turns: Mutex<VecDeque<AssistantTurn>>,
    /// Message counts of the requests received, oldest first.
    seen: Arc<Mutex<Vec<usize>>>,
}

impl Scripted {
    /// Script `turns` and share the recorder with the test.
    pub fn new(turns: Vec<AssistantTurn>) -> (Self, Arc<Mutex<Vec<usize>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                turns: Mutex::new(turns.into_iter().collect()),
                seen: Arc::clone(&seen),
            },
            seen,
        )
    }
}

impl ModelProvider for Scripted {
    fn id(&self) -> &str {
        "mock"
    }

    async fn stream_completion(
        &self,
        _request: &CompletionRequest,
        _on_event: &mut (dyn FnMut(StreamEvent) + Send),
    ) -> VoxResult<()> {
        Ok(())
    }

    async fn complete(&self, request: &CompletionRequest) -> VoxResult<AssistantTurn> {
        self.seen.lock().expect("lock").push(request.messages.len());
        self.turns
            .lock()
            .expect("lock")
            .pop_front()
            .ok_or_else(|| vox_shared::VoxError::Stream {
                detail: "the mock provider ran out of turns".to_string(),
            })
    }
}

/// A turn that reads the seeded file.
pub fn tool_turn() -> AssistantTurn {
    AssistantTurn {
        message: Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: "c1".to_string(),
                name: "read".to_string(),
                arguments: json!({"path": "main.rs"}),
            }],
        },
        finish_reason: Some("tool_calls".to_string()),
        usage: None,
    }
}

/// A turn that ends the run with text.
pub fn text_turn(text: &str) -> AssistantTurn {
    AssistantTurn {
        message: Message::text(Role::Assistant, text),
        finish_reason: Some("stop".to_string()),
        usage: None,
    }
}

/// A project with one seeded file and an agent over it.
pub fn agent(
    dir: &TempDir,
    turns: Vec<AssistantTurn>,
) -> (Agent<Scripted>, Arc<Mutex<Vec<usize>>>) {
    let (provider, seen) = Scripted::new(turns);
    let harness = ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    let agent = Agent::new(
        provider,
        harness,
        &dir.path().join("session.jsonl"),
        "mock-model",
        Limits::new(5, 2),
    )
    .expect("agent");
    (agent, seen)
}

/// A project directory holding one seeded file.
pub fn project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    fs::write(dir.path().join("main.rs"), "alpha\nbeta\n").expect("seed");
    dir
}
