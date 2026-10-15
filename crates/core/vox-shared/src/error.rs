//! Error type shared by every Vox crate.

use std::fmt;

/// Result alias for operations that fail with [`VoxError`].
pub type VoxResult<T> = Result<T, VoxError>;

/// Errors produced by Vox components.
///
/// The `Display` text is generic and safe to show to end users; `detail`
/// carries diagnostic context, is written to logs only, and must never be
/// printed to standard output or surfaced to callers outside the process.
#[derive(Debug)]
pub enum VoxError {
    /// Configuration is missing, invalid, or fails validation.
    Config {
        /// Diagnostic detail for the log; never user-facing.
        detail: String,
    },
    /// A required credential was not found in its environment variable.
    Credential {
        /// Diagnostic detail for the log; never user-facing.
        detail: String,
    },
    /// A provider request or response failed.
    Provider {
        /// Diagnostic detail for the log; never user-facing.
        detail: String,
    },
    /// A streaming response violated the expected protocol.
    Stream {
        /// Diagnostic detail for the log; never user-facing.
        detail: String,
    },
    /// The HTTP transport itself failed.
    Http {
        /// Diagnostic detail for the log; never user-facing.
        detail: String,
    },
    /// A local filesystem, process, or runtime operation failed.
    Io(
        /// Wrapped operating-system error.
        std::io::Error,
    ),
}

impl VoxError {
    /// Diagnostic detail for internal logging; never user-facing.
    pub fn detail(&self) -> String {
        match self {
            Self::Config { detail }
            | Self::Credential { detail }
            | Self::Provider { detail }
            | Self::Stream { detail }
            | Self::Http { detail } => detail.clone(),
            Self::Io(error) => error.to_string(),
        }
    }
}

impl fmt::Display for VoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Config { .. } => "configuration error",
            Self::Credential { .. } => "credential not configured",
            Self::Provider { .. } => "provider request failed",
            Self::Stream { .. } => "stream decode failed",
            Self::Http { .. } => "network error",
            Self::Io(_) => "local I/O error",
        };
        f.write_str(message)
    }
}

impl std::error::Error for VoxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for VoxError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
