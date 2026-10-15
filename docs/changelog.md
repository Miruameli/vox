# Changelog

All notable changes to this project are documented in this file. The format is
based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Added

- `vox-session` (platform layer, ADR-0006): durable append-only JSONL
  session storage. One event per line with a monotonic sequence and
  timestamp; messages, tool calls and results, applied edits, undos,
  and session metadata (with a `parent` link for subagents). Writes
  are `O_APPEND` and synced per event; replay tolerates a torn trailing
  write and rejects corruption before it; every string leaf is redacted
  before it reaches disk; session ids are validated as plain file names.
- `vox-engine-tools` (platform layer): the read, edit, and write tools
  over the edit engine. `ToolHarness` dispatches `ToolCall`s against a
  project root with one undo journal: read returns hashed paged views
  with absolute line numbers (capped pages, parity with the engine's
  whole-file view pinned by test); edit deserializes a `HashlinePatch`
  and applies it, surfacing anchor mismatches as model-facing tool
  results; write creates or replaces atomically with a journaled
  pre-image. Unknown tools and path traversal are refused.
- `vox-engine-edit` (platform layer, ADR-0005): hashline edit engine.
  Patches anchor every addressed line to its 64-bit BLAKE3 content
  hash; a stale anchor rejects the whole patch before the file is
  touched. Writes are atomic (temp file, fsync, rename) with an
  append-only JSONL undo journal of pre-images. Path resolution
  refuses traversal outside the project root, and CRLF files keep
  their terminator.
- Repository foundation: MIT license, CI pipeline, dependabot, PR template.
- Architecture decision records ADR-0001 through ADR-0008 covering the
  Rust-native rewrite, crate layout, async stack, provider abstraction,
  hashline edit engine, session storage, configuration layering, and quality
  gates.
- Documentation set: README, contributing, workflow, code of conduct, security
  policy, development and release runbooks.
- Initial Cargo workspace with core layer crates: `vox-shared`, `vox-config`,
  `vox-domain`; provider crate `vox-provider-openai`; and the `vox` CLI
  interface crate with print mode over a local mock server smoke test.
