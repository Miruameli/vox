//! Boundary tests: subagent metadata links, verbatim tool arguments,
//! and refusal of unsafe session ids.

use serde_json::json;
use vox_domain::ToolCall;
use vox_session::{session_path, EventKind, SessionWriter};

#[test]
fn subagent_meta_records_the_parent() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let path = session_path(dir.path(), "child").expect("path");
    let mut session = SessionWriter::open(&path).expect("open");
    session
        .append(EventKind::SessionMeta {
            session_id: "child".to_string(),
            parent: Some("root".to_string()),
        })
        .expect("append");
    let stored = std::fs::read_to_string(&path).expect("read");
    assert!(stored.contains("\"parent\":\"root\""), "{stored}");
}

#[test]
fn tool_call_arguments_are_stored_verbatim() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let path = session_path(dir.path(), "s1").expect("path");
    let mut session = SessionWriter::open(&path).expect("open");
    session
        .append(EventKind::ToolCall {
            call: ToolCall {
                id: "c1".to_string(),
                name: "read".to_string(),
                arguments: json!({"path": "src/main.rs"}),
            },
        })
        .expect("append");
    let stored = std::fs::read_to_string(&path).expect("read");
    assert!(stored.contains("src/main.rs"), "{stored}");
}

#[test]
fn unsafe_session_ids_are_refused() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    assert!(session_path(dir.path(), "../escape").is_err());
    assert!(session_path(dir.path(), "a/b").is_err());
    assert!(session_path(dir.path(), "").is_err());
    assert!(session_path(dir.path(), "ok-id_1").is_ok());
}
