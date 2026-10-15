//! The prompt line: editing, cursor windows, and recall.

use vox_presentation_tui::input::{InputBuffer, Recall};

#[test]
fn insert_and_backspace_edit_one_character_at_the_cursor() {
    let mut input = InputBuffer::new();
    input.insert('é');
    input.insert('x');
    input.backspace();
    assert_eq!(input.text(), "é");
    assert_eq!(input.cursor(), 1);
}

#[test]
fn delete_removes_the_character_at_the_cursor() {
    let mut input = InputBuffer::new();
    input.set_text("abcd");
    input.move_home();
    input.delete();
    assert_eq!(input.text(), "bcd");
    assert_eq!(input.cursor(), 0);
}

#[test]
fn backspace_at_the_start_is_a_no_op() {
    let mut input = InputBuffer::new();
    input.set_text("ab");
    input.move_home();
    input.backspace();
    assert_eq!(input.text(), "ab");
}

#[test]
fn the_view_scrolls_to_keep_the_cursor_visible() {
    let mut input = InputBuffer::new();
    input.set_text("0123456789");
    input.move_home();
    assert_eq!(input.view(4), ("0123".to_string(), 0));
    input.move_end();
    assert_eq!(input.view(4), ("6789".to_string(), 4));
}

#[test]
fn a_short_prompt_renders_whole_with_the_cursor_at_its_end() {
    let mut input = InputBuffer::new();
    input.insert('h');
    input.insert('i');
    assert_eq!(input.view(10), ("hi".to_string(), 2));
}

#[test]
fn take_empties_the_buffer() {
    let mut input = InputBuffer::new();
    input.set_text("one prompt");
    assert_eq!(input.take(), "one prompt");
    assert!(input.is_empty());
}

#[test]
fn recall_steps_back_and_forward_and_restores_the_draft() {
    let mut recall = Recall::new();
    recall.push("first");
    recall.push("second");
    assert_eq!(recall.previous("draft").as_deref(), Some("second"));
    assert_eq!(recall.previous("draft").as_deref(), Some("first"));
    assert_eq!(recall.previous("draft").as_deref(), Some("first"));
    assert_eq!(recall.forward().as_deref(), Some("second"));
    assert_eq!(recall.forward().as_deref(), Some("draft"));
    assert_eq!(recall.forward(), None);
}

#[test]
fn recall_survives_the_oldest_prompt_being_dropped() {
    let mut recall = Recall::new();
    for index in 0..201 {
        recall.push(&format!("prompt {index}"));
    }
    assert_eq!(recall.previous("draft").as_deref(), Some("prompt 200"));
}
