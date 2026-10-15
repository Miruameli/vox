//! Shell tool tests: one binary for the bash tool and for the
//! descendants that can outlive a command.
//!
//! The shared tool fixtures live one level up; the path attribute
//! is how an integration binary shares a module across the
//! crate roots in this directory.

#[path = "../fixtures/mod.rs"]
mod fixtures;

mod bash_daemon;
mod bash_tool;
