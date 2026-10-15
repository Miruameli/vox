//! Event-level tests: the redaction pass that guards stored events,
//! and the JSON shape of every event type.

use vox_domain::ToolResult;
use vox_session::{EventKind, SessionEvent};

fn result_with(content: &str) -> EventKind {
    EventKind::ToolResult {
        result: ToolResult {
            call_id: "c1".to_string(),
            content: content.to_string(),
            is_error: false,
        },
    }
}

#[test]
fn credential_shaped_content_is_redacted_in_the_event() {
    // Assembled from parts so this source stays scanner-clean.
    let literal = ["sk-", "abcdef12345678"].concat();
    let mut event = SessionEvent::now(1, result_with(&literal));
    event.redact();
    let EventKind::ToolResult { result } = &event.kind else {
        panic!("expected a tool result event");
    };
    assert!(!result.content.contains("abcdef12345678"));
    assert!(result.content.contains("[REDACTED]"));
}

#[test]
fn ordinary_content_survives_redaction_unchanged() {
    let mut event = SessionEvent::now(
        1,
        EventKind::EditApplied {
            path: "src/main.rs".to_string(),
            lines_added: 2,
            lines_removed: 1,
        },
    );
    let before = event.clone();
    event.redact();
    assert_eq!(event, before);
}

#[test]
fn events_round_trip_through_json() {
    let event = SessionEvent::now(
        7,
        EventKind::SessionMeta {
            session_id: "abc".to_string(),
            parent: Some("root".to_string()),
        },
    );
    let text = serde_json::to_string(&event).expect("serialize");
    let back: SessionEvent = serde_json::from_str(&text).expect("deserialize");
    assert_eq!(event, back);
    assert!(text.contains("\"type\":\"session_meta\""), "{text}");
}
