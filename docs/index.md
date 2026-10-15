# Vox documentation

Map of the repository documentation. Every significant decision is recorded as
an ADR; every operating procedure has a runbook or governance page.

## Start here / Mulai di sini

- [README.md](../README.md) — product overview and build instructions
- [changelog.md](changelog.md) — released and unreleased changes
- [guide/onboarding.md](guide/onboarding.md) — first-day guide for contributors
- [guide/glossary.md](guide/glossary.md) — shared vocabulary
- [guide/faq.md](guide/faq.md) — frequent questions

## Architecture / Arsitektur

- [adr/foundation/](adr/foundation/) — core decisions
  - [ADR-0001](adr/foundation/0001-rust-native-reimplementation.md) —
    Rust-native reimplementation of the Pi coding agent
  - [ADR-0002](adr/foundation/0002-modular-crate-layout.md) — modular crate
    layout by architectural layer
  - [ADR-0003](adr/foundation/0003-async-runtime-and-http.md) — tokio async
    runtime and HTTP client stack
  - [ADR-0004](adr/foundation/0004-provider-abstraction.md) — provider
    abstraction, OpenAI-compatible first
- [adr/agent-runtime/](adr/agent-runtime/) — runtime decisions
  - [ADR-0005](adr/agent-runtime/0005-edit-engine-hashline.md) — edit engine
    with content-hash anchored patches
  - [ADR-0006](adr/agent-runtime/0006-session-storage.md) — durable session
    storage format
- [adr/operations/](adr/operations/) — operations decisions
  - [ADR-0007](adr/operations/0007-configuration-and-trust.md) — layered
    configuration and project trust model
  - [ADR-0008](adr/operations/0008-quality-gates.md) — quality gates and CI
    pipeline

## Governance / Tata kelola

- [governance/contributing.md](governance/contributing.md) — how to contribute
- [governance/development-workflow.md](governance/development-workflow.md) —
  issue, branch, PR, and merge rules
- [governance/code-of-conduct.md](governance/code-of-conduct.md) — conduct
- [governance/security.md](governance/security.md) — security policy

## Runbooks

- [runbook/development.md](runbook/development.md) — local development loop
- [runbook/release.md](runbook/release.md) — release process

## Constraints / Batasan (enforced in review)

- Max 150 lines of code per file; split or record an ADR when exceeded.
- Max 5 direct files per folder; introduce subfolders when exceeded.
- Max 5 subfolders per folder; split the domain when exceeded.
- No circular dependencies between crates.
- One file one responsibility; one folder one domain.
- Folder names: descriptive kebab-case; no generic names.
