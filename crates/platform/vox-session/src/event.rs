//! Session event types and the redaction pass that guards them.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use vox_domain::{Message, ToolCall, ToolResult};
use vox_shared::redaction;

/// One durable session event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Monotonically increasing sequence number, starting at 1.
    pub seq: u64,
    /// Unix epoch milliseconds when the event was appended.
    pub timestamp_ms: u64,
    /// The typed payload.
    #[serde(flatten)]
    pub kind: EventKind,
}

/// The typed payload of a session event (ADR-0006 event types).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    /// A conversation message.
    Message {
        /// The message itself.
        message: Message,
    },
    /// A tool invocation requested by the model.
    ToolCall {
        /// The call.
        call: ToolCall,
    },
    /// The result of executing a tool call.
    ToolResult {
        /// The result; its content is redacted before storage.
        result: ToolResult,
    },
    /// An edit the engine applied to the workspace.
    EditApplied {
        /// File the edit touched.
        path: String,
        /// Lines inserted or replaced in.
        lines_added: usize,
        /// Lines replaced out or deleted.
        lines_removed: usize,
    },
    /// An undo that restored a previous file state.
    Undo {
        /// File the undo restored.
        path: String,
        /// Sequence number of the edit being undone.
        undone_seq: u64,
    },
    /// Session identity, written once as the first event.
    SessionMeta {
        /// This session's id.
        session_id: String,
        /// Parent session id when this is a subagent transcript.
        parent: Option<String>,
    },
}

impl SessionEvent {
    /// Build an event at the current time.
    pub fn now(seq: u64, kind: EventKind) -> Self {
        Self {
            seq,
            timestamp_ms: now_millis(),
            kind,
        }
    }

    /// Redact every string leaf of the event in place.
    ///
    /// Serde gives a generic walk: convert to a value, redact all
    /// strings, and convert back. Numeric fields (sequence, timestamp,
    /// line counts) and enum tags are not credentials and stay
    /// untouched by the patterns.
    pub fn redact(&mut self) {
        if let Ok(value) = serde_json::to_value(&self.kind) {
            if let Ok(kind) = serde_json::from_value(redact_value(value)) {
                self.kind = kind;
            }
        }
    }
}

/// Recursively redact every string leaf of a JSON value.
fn redact_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) => serde_json::Value::String(redaction::redact(&text)),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(redact_value).collect())
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, redact_value(value)))
                .collect(),
        ),
        other => other,
    }
}

/// Current time as Unix epoch milliseconds.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}
