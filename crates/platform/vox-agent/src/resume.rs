//! Message reconstruction for resume: session events to conversation.

use vox_domain::message::{ContentBlock, Message, Role};
use vox_domain::tool::{ToolCall, ToolResult};
use vox_session::{EventKind, SessionEvent};

/// The domain message answering one tool result.
pub fn tool_result_message(result: &ToolResult) -> Message {
    Message {
        role: Role::Tool,
        content: vec![ContentBlock::ToolResult {
            call_id: result.call_id.clone(),
            content: result.content.clone(),
        }],
    }
}

/// Rebuild the conversation of a stored session, oldest event first.
///
/// The assistant `message` event already carries its tool calls as
/// content blocks, so the separate `tool_call` execution event adds no
/// conversation turn - it is the audit record. Tool results become
/// `tool` role messages; session metadata, edit, and undo events carry
/// no conversation content and are skipped.
///
/// A run that died between appending an assistant tool call and
/// executing it (infrastructure failure, crash) leaves that call
/// unanswered. Every assistant message whose calls have no results
/// below them is dropped with everything after it: the provider must
/// never receive an unanswered tool call, and events after a dead
/// turn cannot be trusted to follow it.
pub fn messages_from_events(events: &[SessionEvent]) -> Vec<Message> {
    let mut messages = Vec::new();
    for event in events {
        match &event.kind {
            EventKind::Message { message } => messages.push(message.clone()),
            EventKind::ToolResult { result } => messages.push(tool_result_message(result)),
            EventKind::ToolCall { .. }
            | EventKind::SessionMeta { .. }
            | EventKind::EditApplied { .. }
            | EventKind::Undo { .. } => {}
        }
    }
    drop_unanswered_tool_calls(&mut messages);
    messages
}

/// Cut the tail from the last assistant message carrying tool calls
/// that no later tool result answers.
fn drop_unanswered_tool_calls(messages: &mut Vec<Message>) {
    let mut answered: Vec<String> = Vec::new();
    for index in (0..messages.len()).rev() {
        let calls = |block: &ContentBlock| match block {
            ContentBlock::ToolCall { id, .. } => Some(id.clone()),
            _ => None,
        };
        let ids: Vec<String> = messages[index].content.iter().filter_map(calls).collect();
        let results: Vec<String> = messages[index]
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolResult { call_id, .. } => Some(call_id.clone()),
                _ => None,
            })
            .collect();
        answered.extend(results);
        if ids.iter().any(|id| !answered.contains(id)) {
            // The dead turn and everything after it cannot be trusted
            // to follow a call that never ran.
            messages.truncate(index);
            return;
        }
    }
}

/// Tool calls carried by an assistant message.
pub fn tool_calls_of(message: &Message) -> Vec<ToolCall> {
    message
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::ToolCall {
                id,
                name,
                arguments,
            } => Some(ToolCall {
                id: id.clone(),
                name: name.clone(),
                arguments: arguments.clone(),
            }),
            _ => None,
        })
        .collect()
}
