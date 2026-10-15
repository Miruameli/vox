//! OpenAI-compatible streaming provider, non-streaming completion with
//! tool calling, and the wire mapping that carries both.
//!
//! The wire shapes are internal: consumers of this crate speak the
//! [`vox_domain`] contract (messages, tool calls) and own nothing but
//! [`OpenAiClient`] and its [`ModelProvider`](vox_domain::provider::ModelProvider)
//! implementation.

mod client;
mod consume;
mod stream;
mod wire;

pub use client::OpenAiClient;

/// Default base URL for the OpenAI API.
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
