//! Shared primitives used by every Vox crate.
//!
//! This crate is the bottom layer of the workspace: it depends on no other
//! Vox crate and provides the error type, logging, secret redaction, and
//! platform path resolution that all other layers build upon.

pub mod error;
pub mod logging;
pub mod paths;
pub mod redaction;

pub use error::{VoxError, VoxResult};
