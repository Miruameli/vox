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
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox_domain::tool::ToolResult;

    #[test]
    fn meta_and_edit_events_carry_no_conversation() {
        let events = vec![
            SessionEvent::now(
                1,
                EventKind::SessionMeta {
                    session_id: "s".to_string(),
                    parent: None,
                },
            ),
            SessionEvent::now(
                2,
                EventKind::EditApplied {
                    path: "a.txt".to_string(),
                    lines_added: 1,
                    lines_removed: 0,
                },
            ),
            SessionEvent::now(
                3,
                EventKind::Message {
                    message: Message::user("hi"),
                },
            ),
        ];
        let messages = messages_from_events(&events);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text_content(), "hi");
    }

    #[test]
    fn tool_results_become_tool_role_messages() {
        let result = ToolResult::success("c1", "done");
        let message = tool_result_message(&result);
        assert_eq!(message.role, Role::Tool);
        match &message.content[0] {
            ContentBlock::ToolResult { call_id, content } => {
                assert_eq!(call_id, "c1");
                assert_eq!(content, "done");
            }
            other => panic!("unexpected: {other:?}"),
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

#[cfg(test)]
mod tool_call_tests {
    use super::*;

    #[test]
    fn assistant_tool_calls_are_extracted_in_order() {
        let message = Message {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text { text: "hi".into() },
                ContentBlock::ToolCall {
                    id: "c1".into(),
                    name: "read".into(),
                    arguments: serde_json::json!({"path": "a"}),
                },
                ContentBlock::ToolCall {
                    id: "c2".into(),
                    name: "write".into(),
                    arguments: serde_json::json!({"path": "b"}),
                },
            ],
        };
        let calls = tool_calls_of(&message);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "read");
        assert_eq!(calls[1].id, "c2");
    }
}
