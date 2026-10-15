//! Shared fixtures: a scripted mock provider and project scaffolding.

use std::collections::VecDeque;
use std::fs;
use std::sync::Mutex;

use serde_json::json;
use tempfile::TempDir;
use vox_agent::{Agent, Limits};
use vox_domain::completion::{AssistantTurn, CompletionRequest, StreamEvent};
use vox_domain::message::{ContentBlock, Message, Role};
use vox_domain::provider::ModelProvider;
use vox_domain::tool::ToolCall;
use vox_engine_tools::ToolHarness;
use vox_shared::VoxResult;

/// A provider replaying scripted turns, then failing loudly.
pub struct MockProvider {
    turns: Mutex<VecDeque<AssistantTurn>>,
}

impl MockProvider {
    /// Script the turns in order.
    pub fn scripted(turns: Vec<AssistantTurn>) -> Self {
        Self {
            turns: Mutex::new(turns.into_iter().collect()),
        }
    }
}

impl ModelProvider for MockProvider {
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

    async fn complete(&self, _request: &CompletionRequest) -> VoxResult<AssistantTurn> {
        self.turns
            .lock()
            .expect("lock")
            .pop_front()
            .ok_or_else(|| vox_shared::VoxError::Stream {
                detail: "mock provider ran out of turns".to_string(),
            })
    }
}

/// A turn requesting the given tool calls.
pub fn tool_call_turn(calls: &[ToolCall]) -> AssistantTurn {
    AssistantTurn {
        message: Message {
            role: Role::Assistant,
            content: calls
                .iter()
                .map(|call| ContentBlock::ToolCall {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                })
                .collect(),
        },
        finish_reason: Some("tool_calls".to_string()),
        usage: None,
    }
}

/// A turn that ends the run with text.
pub fn stop_turn(text: &str) -> AssistantTurn {
    AssistantTurn {
        message: Message::text(Role::Assistant, text),
        finish_reason: Some("stop".to_string()),
        usage: None,
    }
}

/// A read call against the seeded project file.
pub fn read_call() -> ToolCall {
    ToolCall {
        id: "c1".to_string(),
        name: "read".to_string(),
        arguments: json!({"path": "main.rs"}),
    }
}

/// A project directory with one seeded file.
pub fn project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    fs::write(dir.path().join("main.rs"), "alpha\nbeta\n").expect("seed");
    dir
}

/// An agent over the project with a scripted provider.
pub fn agent(dir: &TempDir, turns: Vec<AssistantTurn>) -> Agent<MockProvider> {
    let harness = ToolHarness::new(dir.path(), &dir.path().join("undo.jsonl")).expect("harness");
    Agent::new(
        MockProvider::scripted(turns),
        harness,
        &dir.path().join("s.jsonl"),
        "mock-model",
        Limits::new(5, 2),
    )
    .expect("agent")
}
