//! Strict validation of a fully layered configuration.

use vox_shared::{VoxError, VoxResult};

use crate::schema::Config;

/// Accepted log filters.
const LOG_LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];

/// Validate a fully layered configuration.
///
/// Every failure carries a detail string for the log; the user-facing
/// message stays generic (see [`VoxError::Config`]).
pub(super) fn validate(config: &Config) -> VoxResult<()> {
    let general = &config.general;
    if general.provider.trim().is_empty() {
        return Err(config_error("provider must not be empty"));
    }
    if general.model.trim().is_empty() {
        return Err(config_error("model must not be empty"));
    }
    if !LOG_LEVELS.contains(&general.log.as_str()) {
        return Err(config_error(&format!("unknown log filter {}", general.log)));
    }
    if !is_env_var_name(&general.api_key_env) {
        return Err(config_error(
            "api_key_env must be an environment variable name",
        ));
    }
    if let Some(base_url) = &general.base_url {
        if !is_allowed_base_url(base_url) {
            return Err(config_error("base_url must be https, or http on loopback"));
        }
    }
    Ok(())
}

/// Build a config error with a generic user-facing message.
fn config_error(detail: &str) -> VoxError {
    VoxError::Config {
        detail: detail.to_string(),
    }
}

/// Report whether `name` is a valid environment variable name.
fn is_env_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Report whether `url` is an allowed provider base URL.
///
/// HTTPS is always allowed; plain HTTP is restricted to loopback hosts so
/// credentials never leave the machine in clear text.
fn is_allowed_base_url(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if url.chars().any(char::is_whitespace) {
        return false;
    }
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .to_lowercase();
    let host = host.split('@').next_back().unwrap_or(&host);
    let host = host.split(':').next().unwrap_or(host);
    match scheme {
        "https" => !host.is_empty(),
        "http" => {
            host == "localhost"
                || host == "::1"
                || host == "[::1]"
                || host == "127.0.0.1"
                || host.starts_with("127.")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Config;

    #[test]
    fn hashes_and_ports_are_not_hosts() {
        assert!(is_allowed_base_url("https://example.com/v1"));
        assert!(!is_allowed_base_url("https:///v1"));
        assert!(is_allowed_base_url("http://127.0.0.1:11434/v1"));
        assert!(is_allowed_base_url("http://localhost:1234"));
        assert!(!is_allowed_base_url("http://10.0.0.5:1234"));
        assert!(!is_allowed_base_url("ftp://example.com"));
        assert!(!is_allowed_base_url("example.com"));
        assert!(!is_allowed_base_url("https://exa mple.com"));
    }

    #[test]
    fn env_var_names_are_validated() {
        assert!(is_env_var_name("OPENAI_API_KEY"));
        assert!(is_env_var_name("_private1"));
        assert!(!is_env_var_name("1BAD"));
        assert!(!is_env_var_name("BAD-NAME"));
        assert!(!is_env_var_name(""));
    }

    #[test]
    fn empty_fields_are_rejected() {
        let mut config = Config::default();
        config.general.provider = "  ".into();
        let error = validate(&config).expect_err("reject");
        assert!(error.detail().contains("provider"));
    }
}
