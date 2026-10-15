//! Wrapping and display lines: one place decides where a line ends.

use vox_presentation_tui::app::transcript::{Entry, EntryKind, Transcript};
use vox_presentation_tui::render::wrap::{display_lines, wrap_lines};

#[test]
fn wrapping_keeps_words_whole() {
    assert_eq!(wrap_lines("one two three", 7), vec!["one two", "three"]);
}

#[test]
fn wrapping_honours_explicit_newlines() {
    assert_eq!(wrap_lines("a\nb", 200), vec!["a", "b"]);
}

#[test]
fn wrapping_splits_a_word_longer_than_the_width() {
    assert_eq!(wrap_lines("abcdef", 3), vec!["abc", "def"]);
}

#[test]
fn wrapping_measures_columns_not_characters() {
    assert_eq!(
        wrap_lines("日本語テキスト", 6),
        vec!["日本語", "テキス", "ト"]
    );
}

#[test]
fn a_trailing_newline_keeps_an_empty_last_line() {
    assert_eq!(wrap_lines("a\n", 200), vec!["a", ""]);
}

#[test]
fn leading_indentation_is_kept_on_the_first_line() {
    assert_eq!(wrap_lines("    indented", 40), vec!["    indented"]);
    assert_eq!(wrap_lines("  one two three", 9), vec!["  one two", "three"]);
}

#[test]
fn a_long_word_is_split_and_the_next_word_fills_the_remainder() {
    assert_eq!(wrap_lines("abcdefgh", 3), vec!["abc", "def", "gh"]);
    assert_eq!(wrap_lines("abcdefgh ij", 3), vec!["abc", "def", "gh", "ij"]);
}

#[test]
fn every_line_fits_the_width_it_was_wrapped_at() {
    let text = "the quick brown fox jumps over the lazy dog, then keeps running";
    for width in 1..20 {
        for line in wrap_lines(text, width) {
            assert!(line.chars().count() <= width, "{line:?} at width {width}");
        }
    }
}

#[test]
fn display_lines_separate_entries_and_mark_their_first_line() {
    let mut transcript = Transcript::new();
    transcript.push(Entry::new(EntryKind::User, "one"));
    transcript.push(Entry::new(EntryKind::Assistant, "two"));
    let lines = display_lines(&transcript, 40, false);
    let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(texts, vec!["one", "", "two"]);
    assert!(lines[0].first);
    assert!(!lines[1].first);
    assert!(lines[2].first);
}

#[test]
fn long_entries_wrap_and_only_the_first_line_is_marked() {
    let mut transcript = Transcript::new();
    transcript.push(Entry::new(EntryKind::User, "alpha beta gamma"));
    let lines = display_lines(&transcript, 11, false);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].first);
    assert!(!lines[1].first);
}

#[test]
fn detail_is_hidden_until_expanded_and_is_indented_when_shown() {
    let mut transcript = Transcript::new();
    transcript.push(Entry::with_detail(
        EntryKind::Tool,
        "✓ read",
        "path: main.rs",
    ));
    let collapsed = display_lines(&transcript, 40, false);
    let expanded = display_lines(&transcript, 40, true);
    assert_eq!(collapsed.len(), 1);
    assert_eq!(expanded.len(), 2);
    assert_eq!(expanded[1].text, "  path: main.rs");
}

#[test]
fn the_transcript_drops_the_oldest_entry_past_the_limit() {
    let mut transcript = Transcript::new();
    for index in 0..2000 {
        transcript.push(Entry::new(EntryKind::System, index.to_string()));
    }
    transcript.push(Entry::new(EntryKind::System, "newest"));
    assert_eq!(transcript.entries().len(), 2000);
    assert_eq!(transcript.entries().last().unwrap().text, "newest");
}
