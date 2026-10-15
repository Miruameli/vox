//! OpenAI-compatible streaming provider adapter.

pub mod client;
mod consume;
pub mod stream;
pub mod wire;

pub use client::OpenAiClient;

/// Default base URL for the OpenAI API.
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
