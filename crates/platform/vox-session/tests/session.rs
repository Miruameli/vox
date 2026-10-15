//! Writer-reader round trips: append, replay, sequence continuity
//! across reopens, torn writes, and the no-secrets-on-disk guarantee.

use std::fs;
use std::io::Write;

use tempfile::TempDir;
use vox_domain::{Message, Role, ToolResult};
use vox_session::{replay, session_path, EventKind, SessionEvent, SessionWriter};

fn scratch(name: &str) -> TempDir {
    let dir = TempDir::new().unwrap_or_else(|_| panic!("temp dir {name}"));
    dir
}

fn writer(dir: &TempDir) -> SessionWriter {
    SessionWriter::open(&session_path(dir.path(), "s1").expect("path")).expect("open")
}

fn read_back(dir: &TempDir) -> Vec<SessionEvent> {
    let path = session_path(dir.path(), "s1").expect("path");
    replay(&fs::read_to_string(path).expect("read")).expect("replay")
}

#[test]
fn append_and_replay_round_trip() {
    let dir = scratch("round-trip");
    let mut session = writer(&dir);
    session
        .append(EventKind::Message {
            message: Message::user("hello"),
        })
        .expect("append");
    session
        .append(EventKind::ToolResult {
            result: ToolResult::success("c1", "read 3 lines"),
        })
        .expect("append");
    let events = read_back(&dir);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].seq, 1);
    assert!(matches!(events[0].kind, EventKind::Message { .. }));
    assert!(matches!(events[1].kind, EventKind::ToolResult { .. }));
}

#[test]
fn sequence_continues_across_reopens() {
    let dir = scratch("reopen");
    {
        let mut session = writer(&dir);
        session
            .append(EventKind::SessionMeta {
                session_id: "s1".to_string(),
                parent: None,
            })
            .expect("append");
    }
    let mut reopened = writer(&dir);
    let event = reopened
        .append(EventKind::Message {
            message: Message::text(Role::Assistant, "continued"),
        })
        .expect("append");
    assert_eq!(event.seq, 2);
}

#[test]
fn torn_trailing_write_is_tolerated_and_its_sequence_is_reused() {
    let dir = scratch("torn");
    let path = session_path(dir.path(), "s1").expect("path");
    {
        let mut session = writer(&dir);
        session
            .append(EventKind::Message {
                message: Message::user("committed"),
            })
            .expect("append");
    }
    // Simulate a crash mid-append: a partial line without its newline.
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("open")
        .write_all(b"{\"seq\":2,\"timestamp_ms\":1,\"type\":\"mess")
        .expect("simulate torn write");

    let events = read_back(&dir);
    assert_eq!(events.len(), 1);

    let mut session = writer(&dir);
    let next = session
        .append(EventKind::Message {
            message: Message::text(Role::User, "after crash"),
        })
        .expect("append");
    assert_eq!(next.seq, 2);
}

#[test]
fn tool_result_secrets_never_reach_the_file() {
    let dir = scratch("redaction");
    let path = session_path(dir.path(), "s1").expect("path");
    let literal = ["sk-", "abcdef12345678"].concat();
    let mut session = writer(&dir);
    session
        .append(EventKind::ToolResult {
            result: ToolResult::success("c1", &literal),
        })
        .expect("append");

    let stored = fs::read_to_string(&path).expect("read");
    assert!(
        !stored.contains("abcdef12345678"),
        "secret on disk: {stored}"
    );
    assert!(stored.contains("[REDACTED]"));
    let events = read_back(&dir);
    assert_eq!(events.len(), 1);
}
