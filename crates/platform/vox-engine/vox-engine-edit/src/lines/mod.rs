//! The line text model: per-line terminators and splice arithmetic.
//!
//! Text is held as lines that each carry their own terminator, so a
//! file with mixed `\n` and `\r\n` endings round-trips byte-for-byte
//! and keeps the endings of lines a patch does not touch. Patch
//! application works top-down through the highest anchor first, so
//! every splice sees the buffer in the coordinates the anchors were
//! validated against.

use crate::patch::EditOp;

/// One line of text: its content and its own terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextLine {
    /// Line content without any terminator.
    pub content: String,
    /// Terminator: `"\n"`, `"\r\n"`, or `""` for a final line with no
    /// trailing newline.
    pub terminator: &'static str,
}

/// Split text into lines carrying their own terminators.
///
/// The concatenation of every content plus terminator reproduces the
/// input byte-for-byte, including mixed `\n` and `\r\n` endings and a
/// missing final newline. An empty text yields no lines.
pub fn split_lines(text: &str) -> Vec<TextLine> {
    let mut lines = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        match rest.find('\n') {
            None => {
                lines.push(TextLine {
                    content: rest.to_string(),
                    terminator: "",
                });
                break;
            }
            Some(index) => {
                let carriage = index > 0 && rest.as_bytes()[index - 1] == b'\r';
                let content = if carriage {
                    &rest[..index - 1]
                } else {
                    &rest[..index]
                };
                lines.push(TextLine {
                    content: content.to_string(),
                    terminator: if carriage { "\r\n" } else { "\n" },
                });
                rest = &rest[index + 1..];
            }
        }
    }
    lines
}

/// Rebuild text from lines, each with its own terminator.
pub fn join_lines(lines: &[TextLine]) -> String {
    lines
        .iter()
        .map(|line| line.content.clone() + line.terminator)
        .collect()
}

/// Terminator of the line above the anchor, or `"\n"` for the first
/// line of the file: what an unterminated final line lends its
/// neighbors.
fn previous_terminator(lines: &[TextLine], line: usize) -> &'static str {
    line.checked_sub(2)
        .and_then(|index| lines.get(index))
        .map(|previous| previous.terminator)
        .unwrap_or("\n")
}

/// Apply validated, anchor-checked ops to the line buffer, highest
/// anchor first, and return `(lines added, lines removed)`.
///
/// Inserted and replacement content borrows the terminator of the edit
/// site, so a `\r\n` line yields `\r\n`-terminated content. A final
/// line without a trailing newline lends its position (not a
/// fabricated ending): replacing it makes the block's last line the
/// new unterminated final line; inserting after it repairs the
/// anchor's missing terminator while the block's last line stays
/// unterminated, so content never fuses.
pub(crate) fn apply_ops(lines: &mut Vec<TextLine>, ops: &[EditOp]) -> (usize, usize) {
    let mut ordered: Vec<&EditOp> = ops.iter().collect();
    ordered.sort_by_key(|op| std::cmp::Reverse(op.anchor_line()));
    let (mut added, mut removed) = (0usize, 0usize);
    for op in ordered {
        let line = op.anchor_line();
        let (middle, last) = match op {
            EditOp::Replace { .. } => {
                let anchor = lines[line - 1].terminator;
                let middle = if anchor.is_empty() {
                    previous_terminator(lines, line)
                } else {
                    anchor
                };
                (middle, anchor)
            }
            EditOp::Insert { .. } => {
                if lines[line - 1].terminator.is_empty() {
                    let middle = previous_terminator(lines, line);
                    lines[line - 1].terminator = middle;
                    (middle, "")
                } else {
                    let taken = lines[line - 1].terminator;
                    (taken, taken)
                }
            }
            EditOp::Delete { .. } => ("\n", ""),
        };
        match op {
            EditOp::Replace { new_lines, .. } => {
                let block = splice_block(new_lines, middle, last);
                added += new_lines.len();
                removed += 1;
                lines.splice(line - 1..line, block);
            }
            EditOp::Insert { new_lines, .. } => {
                let block = splice_block(new_lines, middle, last);
                added += new_lines.len();
                lines.splice(line..line, block);
            }
            EditOp::Delete { .. } => {
                lines.remove(line - 1);
                removed += 1;
            }
        }
    }
    (added, removed)
}

/// Wrap replacement content in lines carrying the file's local ending.
fn splice_block(content: &[String], middle: &'static str, last: &'static str) -> Vec<TextLine> {
    content
        .iter()
        .enumerate()
        .map(|(index, text)| TextLine {
            content: text.clone(),
            terminator: if index + 1 == content.len() {
                last
            } else {
                middle
            },
        })
        .collect()
}
