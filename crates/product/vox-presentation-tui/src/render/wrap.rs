//! Wrapping: entry text to display lines.
//!
//! [`wrap_lines`] is the one place that decides where a line ends, and
//! [`display_lines`] is the one place that decides which lines an entry
//! occupies - one line collapsed, more when tool detail is expanded.
//! Both are pure functions of their arguments, so the view and its
//! tests agree on exactly what the terminal shows, and no rendering
//! state has to be cached in the application state.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::transcript::{Entry, EntryKind, Transcript};

/// One display line: the entry it came from and its wrapped text.
pub struct DisplayLine {
    /// Entry this line came from, which decides its style.
    pub kind: EntryKind,
    /// Wrapped text, already fitted to the width it was wrapped at.
    pub text: String,
    /// Whether this is the first line of its entry, which is the only
    /// line that carries the label.
    pub first: bool,
}

/// Build one display line.
fn line(kind: EntryKind, text: String, first: bool) -> DisplayLine {
    DisplayLine { kind, text, first }
}

/// Display lines of the whole transcript at `width`.
///
/// A blank line separates entries, so the transcript reads as a
/// conversation instead of a wall of text. `expanded` reveals the
/// detail every entry carries, indented under its first line.
pub fn display_lines(transcript: &Transcript, width: usize, expanded: bool) -> Vec<DisplayLine> {
    let mut lines = Vec::new();
    for (index, entry) in transcript.entries().iter().enumerate() {
        if index > 0 {
            lines.push(line(EntryKind::System, String::new(), false));
        }
        lines.extend(entry_lines(entry, width, expanded));
    }
    lines
}

/// Display lines of one entry at `width`.
fn entry_lines(entry: &Entry, width: usize, expanded: bool) -> Vec<DisplayLine> {
    let mut lines: Vec<DisplayLine> = wrap_lines(entry.text.trim_end(), width)
        .into_iter()
        .enumerate()
        .map(|(index, text)| line(entry.kind, text, index == 0))
        .collect();
    if expanded {
        if let Some(detail) = &entry.detail {
            let indented = format!("  {detail}");
            lines.extend(
                wrap_lines(&indented, width)
                    .into_iter()
                    .map(|text| line(entry.kind, text, false)),
            );
        }
    }
    lines
}

/// Wrap `text` into display lines of at most `width` columns.
///
/// Explicit newlines are honoured and a trailing newline produces a
/// trailing empty line, so wrapped text keeps the shape the author
/// wrote. A run of leading spaces is kept as an indent on the first
/// line of its source line, so indented output (code, lists) stays
/// recognisable. Words longer than `width` are split on character
/// boundaries so no content is lost. Width is measured in terminal
/// columns, not characters: a wide (CJK) character occupies two, which
/// is what the renderer clips against. Combining marks inside a word
/// are counted with the character that carries them, so the count is a
/// column-accurate approximation rather than a grapheme measurement.
pub fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        let (prefix, body) = raw.split_at(indent);
        let mut wrapped = wrap_words(body, width.saturating_sub(indent));
        // The indent rides on the first line, which is what leaves the
        // rest of the source line the full width to wrap into.
        if let Some(first) = wrapped.first_mut() {
            first.insert_str(0, prefix);
        }
        lines.extend(wrapped);
    }
    lines
}

/// Wrap one source line of words to at most `width` columns.
///
/// Returns at least one line, even for an empty one, so a caller that
/// maps lines one to one never loses a line it was given.
fn wrap_words(raw: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for word in raw.split(' ') {
        let separator = usize::from(!current.is_empty());
        let word_width = word.width();
        if current_width + separator + word_width <= width {
            if separator == 1 {
                current.push(' ');
            }
            current.push_str(word);
            current_width += separator + word_width;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }
        if word_width <= width {
            current.push_str(word);
            current_width = word_width;
            continue;
        }
        // A word wider than the line is split across lines; the last
        // chunk continues on the current line so the next word may
        // still fit beside it.
        let mut chunks = split_long_word(word, width);
        if let Some(last) = chunks.pop() {
            current = last;
            current_width = current.width();
        }
        lines.extend(chunks);
    }
    lines.push(current);
    lines
}

/// Split one word into column-sized chunks.
fn split_long_word(word: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut chunk = String::new();
    let mut chunk_width = 0usize;
    for character in word.chars() {
        if chunk_width + character_width(character) > width {
            chunks.push(std::mem::take(&mut chunk));
            chunk_width = 0;
        }
        chunk_width += character_width(character);
        chunk.push(character);
    }
    chunks.push(chunk);
    chunks
}

/// Display width of one character, counting zero-width marks as 0.
fn character_width(character: char) -> usize {
    UnicodeWidthChar::width(character).unwrap_or(0)
}
