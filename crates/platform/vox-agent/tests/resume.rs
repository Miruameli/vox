//! Resume reconstruction: session events to conversation, and the
//! dead-turn truncation that keeps the rebuild well-formed.

use serde_json::json;

use vox_agent::messages_from_events;
use vox_domain::message::{ContentBlock, Message, Role};
use vox_domain::tool::ToolResult;
use vox_session::{EventKind, SessionEvent};

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
fn an_unanswered_tool_call_drops_the_dead_tail() {
    // A run that died between appending the assistant call and
    // executing it must not resume with an unanswered call.
    let events = vec![
        SessionEvent::now(
            1,
            EventKind::Message {
                message: Message::user("go"),
            },
        ),
        SessionEvent::now(
            2,
            EventKind::Message {
                message: Message {
                    role: Role::Assistant,
                    content: vec![ContentBlock::Text { text: "ok".into() }],
                },
            },
        ),
        SessionEvent::now(
            3,
            EventKind::Message {
                message: Message {
                    role: Role::Assistant,
                    content: vec![ContentBlock::ToolCall {
                        id: "c1".to_string(),
                        name: "read".to_string(),
                        arguments: json!({"path": "a"}),
                    }],
                },
            },
        ),
    ];
    let messages = messages_from_events(&events);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].role, Role::Assistant);

    // The same session with the tool result answering the call: the
    // whole conversation survives, nothing is truncated.
    let mut full = events;
    full.push(SessionEvent::now(
        4,
        EventKind::ToolResult {
            result: ToolResult::success("c1", "file"),
        },
    ));
    let answered = messages_from_events(&full);
    assert_eq!(answered.len(), 4);
    assert_eq!(answered[3].role, Role::Tool);
}
