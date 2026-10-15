//! Content-hash anchoring for line-addressed edits.

use blake3::Hasher;

/// Anchor hash length in hex characters.
///
/// The 256-bit BLAKE3 digest is truncated to 64 bits: collisions need
/// ~2^32 lines inside one file, far beyond any editable source file.
pub const HASH_LEN: usize = 16;

/// Compute the anchor hash of one line.
///
/// The line terminator is excluded, so CRLF and LF files anchor
/// identically.
pub fn line_hash(content: &str) -> String {
    let content = content.strip_suffix('\n').unwrap_or(content);
    let content = content.strip_suffix('\r').unwrap_or(content);
    let mut hasher = Hasher::new();
    hasher.update(content.as_bytes());
    hasher.finalize().to_hex()[..HASH_LEN].to_string()
}

/// Whether `content` still matches the anchor `expected`.
pub fn anchor_matches(expected: &str, content: &str) -> bool {
    line_hash(content) == expected
}

/// Render the hashed view of a file for the read tool.
///
/// Each line becomes `number|hash|content`, letting the model address
/// lines by hash without retyping context. The trailing newline of the
/// last line is dropped from the view.
pub fn hashed_view(file: &str) -> String {
    let mut view = String::new();
    for (index, line) in file.lines().enumerate() {
        view.push_str(&format!("{:>5}|{}|{line}\n", index + 1, line_hash(line)));
    }
    view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_sixteen_lowercase_hex_chars_and_stable() {
        let hash = line_hash("fn main() {}");
        assert_eq!(hash.len(), HASH_LEN);
        assert!(hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert_eq!(hash, line_hash("fn main() {}"));
        assert_ne!(hash, line_hash("fn main() { }"));
    }

    #[test]
    fn line_terminator_is_excluded_from_the_hash() {
        assert_eq!(line_hash("abc\n"), line_hash("abc"));
        assert_eq!(line_hash("abc\r\n"), line_hash("abc"));
    }

    #[test]
    fn hashed_view_prefixes_number_and_hash() {
        let view = hashed_view("one\ntwo");
        let first = view.lines().next().expect("one line");
        let (number, rest) = first.split_once('|').expect("number separator");
        let (hash, content) = rest.split_once('|').expect("hash separator");
        assert_eq!(number.trim(), "1");
        assert_eq!(content, "one");
        assert_eq!(hash, line_hash("one"));
        assert_eq!(view.lines().count(), 2);
    }
}
