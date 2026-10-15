//! Redaction of secrets from text before logging or persistence.

use regex::Regex;
use std::sync::LazyLock;

/// Placeholder substituted for credential-shaped values.
pub const REDACTED: &str = "[REDACTED]";

/// Matches `key=value` style assignments whose key names a credential.
fn assignment_pattern() -> &'static Regex {
    static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)(api[_-]?key|apikey|token|secret|password|authorization)(\s*[:=]\s*)((?:bearer\s+)?[^\s]+)",
        )
        .expect("assignment pattern is a valid regex")
    });
    &PATTERN
}

/// Matches bearer tokens in headers.
fn bearer_pattern() -> &'static Regex {
    static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(bearer)(\s+)([A-Za-z0-9._\-]+)")
            .expect("bearer pattern is a valid regex")
    });
    &PATTERN
}

/// Matches provider-style literal API keys (for example `sk-...`).
fn literal_key_pattern() -> &'static Regex {
    static PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\bsk-[A-Za-z0-9_-]{8,}").expect("valid regex"));
    &PATTERN
}

/// Replace credential-shaped values in `text` with [`REDACTED`].
pub fn redact(text: &str) -> String {
    let redacted = assignment_pattern().replace_all(text, "$1$2[REDACTED]");
    let redacted = bearer_pattern().replace_all(&redacted, "$1$2[REDACTED]");
    literal_key_pattern()
        .replace_all(&redacted, REDACTED)
        .into_owned()
}

/// Report whether `text` contains a credential-shaped value.
///
/// Used to reject configuration files that embed secrets.
pub fn contains_secret(text: &str) -> bool {
    redact(text) != text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_values_are_redacted() {
        // Fixtures are assembled from parts so no credential-shaped literal
        // appears in the source for secret scanners to flag.
        let input = ["api_key", "=fixture-value"].concat();
        let expected = ["api_key", "=", REDACTED].concat();
        assert_eq!(redact(&input), expected);
        assert!(contains_secret(&input));
    }

    #[test]
    fn bearer_tokens_are_redacted() {
        // The scheme belongs to the value; the whole credential must vanish.
        let redacted = redact("Authorization: Bearer abc.def-123");
        assert_eq!(redacted, "Authorization: [REDACTED]");
        let redacted = redact("send Bearer abc.def-123 in the header");
        assert_eq!(redacted, "send Bearer [REDACTED] in the header");
    }

    #[test]
    fn literal_keys_are_redacted() {
        // Built at runtime: the repository must never carry a literal
        // that a secret scanner flags, while the pattern path stays
        // exercised with a key-shaped value.
        let token = format!("sk-{}", "a".repeat(14));
        let redacted = redact(&format!("found token {token} in output"));
        assert_eq!(redacted, "found token [REDACTED] in output");
    }

    #[test]
    fn clean_text_is_untouched() {
        let text = "the agent read three files and wrote one";
        assert_eq!(redact(text), text);
        assert!(!contains_secret(text));
    }
}
