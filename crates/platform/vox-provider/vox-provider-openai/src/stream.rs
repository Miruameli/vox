//! Incremental Server-Sent-Events decoder.

use vox_shared::{VoxError, VoxResult};

/// Decodes an SSE byte stream into event data payloads.
///
/// The decoder buffers partial input so feeds may split anywhere, including
/// inside an event. Field lines other than `data` (comments, `event:`,
/// `id:`) are ignored per the SSE specification.
#[derive(Debug, Default)]
pub struct SseDecoder {
    /// Bytes received but not yet terminated by a blank line.
    buffer: String,
}

impl SseDecoder {
    /// Create an empty decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed bytes and return every completed event's `data` payload.
    ///
    /// Events are terminated by a blank line; both `LF` and `CRLF` line
    /// endings are accepted.
    pub fn push(&mut self, bytes: &[u8]) -> VoxResult<Vec<String>> {
        let text = String::from_utf8_lossy(bytes);
        if text.contains('\u{FFFD}') && bytes.iter().any(|&b| b >= 0x80) {
            return Err(VoxError::Stream {
                detail: "stream chunk is not valid UTF-8".to_string(),
            });
        }
        self.buffer.push_str(&text);
        let mut events = Vec::new();
        while let Some(index) = find_event_end(&self.buffer) {
            let raw = self.buffer[..index].to_string();
            self.buffer
                .drain(..index + separator_len(&self.buffer, index));
            if let Some(payload) = event_data(&raw) {
                events.push(payload);
            }
        }
        Ok(events)
    }
}

/// Index of the first character of the blank line ending an event.
fn find_event_end(buffer: &str) -> Option<usize> {
    let bytes = buffer.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] == b'\n' {
            let next_is_blank = match bytes.get(index + 1) {
                Some(b'\n') => true,
                Some(b'\r') if bytes.get(index + 2) == Some(&b'\n') => true,
                _ => false,
            };
            if next_is_blank {
                return Some(index);
            }
        }
    }
    None
}

/// Length of the blank-line separator starting at `index`.
fn separator_len(buffer: &str, index: usize) -> usize {
    let rest = &buffer[index..];
    if rest.starts_with("\r\n\r\n") {
        4
    } else if rest.starts_with("\n\n") {
        2
    } else {
        1
    }
}

/// Extract the joined `data` payload of one raw event block.
fn event_data(raw: &str) -> Option<String> {
    let mut payload = String::new();
    let mut seen = false;
    for line in raw.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(value) = line.strip_prefix("data:") {
            if seen {
                payload.push('\n');
            }
            payload.push_str(value.strip_prefix(' ').unwrap_or(value));
            seen = true;
        }
    }
    seen.then_some(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_event_in_one_feed() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"data: hello\n\n").expect("push");
        assert_eq!(events, vec!["hello".to_string()]);
    }

    #[test]
    fn event_split_across_feeds() {
        let mut decoder = SseDecoder::new();
        assert!(decoder.push(b"data: par").expect("push").is_empty());
        assert!(decoder.push(b"tial\n").expect("push").is_empty());
        let events = decoder.push(b"\n").expect("push");
        assert_eq!(events, vec!["partial".to_string()]);
    }

    #[test]
    fn crlf_and_comments_are_handled() {
        let mut decoder = SseDecoder::new();
        let events = decoder
            .push(b": keep-alive\r\ndata: one\r\ndata: two\r\n\r\n")
            .expect("push");
        assert_eq!(events, vec!["one\ntwo".to_string()]);
    }

    #[test]
    fn multiple_events_in_one_feed() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"data: a\n\ndata: b\n\n").expect("push");
        assert_eq!(events, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        let mut decoder = SseDecoder::new();
        let error = decoder.push(&[0x80, 0x81]).expect_err("reject");
        assert!(error.detail().contains("UTF-8"));
    }
}
