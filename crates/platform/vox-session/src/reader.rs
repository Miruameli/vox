//! Replay of a session file into events.

use vox_shared::VoxResult;

use crate::event::SessionEvent;

/// Parse one stored line into an event.
pub fn parse_line(line: &str) -> VoxResult<SessionEvent> {
    serde_json::from_str(line).map_err(|error| vox_shared::VoxError::Config {
        detail: format!("session event decode failed: {error}"),
    })
}

/// Replay every committed event of `text`, oldest first.
///
/// A trailing segment that fails to parse is tolerated only when the
/// file lacks its final newline: it is a write a crash lost, and any
/// corruption before the tail is an error - a session file that lies
/// is worse than one that fails.
pub fn replay(text: &str) -> VoxResult<Vec<SessionEvent>> {
    let mut events = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line) {
            Ok(event) => events.push(event),
            Err(error) => {
                let torn_tail = lines.peek().is_none() && !text.ends_with('\n');
                if !torn_tail {
                    return Err(error);
                }
                break;
            }
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventKind;

    fn event(seq: u64) -> SessionEvent {
        SessionEvent::now(
            seq,
            EventKind::SessionMeta {
                session_id: "s".to_string(),
                parent: None,
            },
        )
    }

    #[test]
    fn replay_returns_events_in_order() {
        let mut text = String::new();
        for seq in 1..=3 {
            text.push_str(&serde_json::to_string(&event(seq)).expect("serialize"));
            text.push('\n');
        }
        let events = replay(&text).expect("replay");
        assert_eq!(events.len(), 3);
        assert_eq!(
            events.iter().map(|e| e.seq).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn trailing_partial_line_is_tolerated() {
        let mut text = serde_json::to_string(&event(1)).expect("serialize");
        text.push('\n');
        text.push_str("{\"seq\":2,\"timestamp_ms\":1,\"type\":\"mess");
        let events = replay(&text).expect("replay");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn corruption_before_the_tail_is_an_error() {
        let mut text = "{ not json }\n".to_string();
        text.push_str(&serde_json::to_string(&event(2)).expect("serialize"));
        text.push('\n');
        assert!(replay(&text).is_err());
    }

    #[test]
    fn empty_text_replays_to_no_events() {
        assert!(replay("").expect("replay").is_empty());
        assert!(replay("\n\n").expect("replay").is_empty());
    }
}
