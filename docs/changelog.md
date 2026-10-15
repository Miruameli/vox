# Changelog

All notable changes to this project are documented in this file. The format is
based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Added

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
