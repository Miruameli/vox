# Changelog

All notable changes to this project are documented in this file. The format is
based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Added

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
