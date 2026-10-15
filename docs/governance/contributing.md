# Contributing

Thank you for contributing to Vox. This document explains how to build, test,
and ship changes. Read [development-workflow.md](development-workflow.md) for
the mandatory process around issues, branches, and pull requests.

## Prerequisites

- Rust 1.98 or newer (pinned in CI; see [../runbook/development.md](../runbook/development.md))
- `cargo`, `cargo fmt`, `cargo clippy`
- A GitHub account with the CLI `gh` authenticated

## Repository layout

Cargo workspace organized by architectural layer. One folder one domain, one
file one responsibility. Enforced limits:

| Rule | Limit |
| --- | --- |
| Lines of code per file | 150 (split or write an ADR above this) |
| Direct files per folder | 5 (introduce subfolders above this) |
| Subfolders per folder | 5 (split the domain above this) |
| Cross-crate dependencies | acyclic, documented in ADRs |

Layer map (see [ADR-0002](../adr/foundation/0002-modular-crate-layout.md)):

| Layer | Crates |
| --- | --- |
| domain | `vox-domain` |
| application | `vox-application` |
| shared | `vox-shared` |
| config | `vox-config` |
| infrastructure | `vox-provider-*`, `vox-engine-*`, `vox-runtime-*` |
| presentation | `vox-presentation-tui` |
| interfaces | `vox-interfaces-cli`, `vox-interface-rpc` |

## Local loop

```sh
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the CLI from source:

```sh
cargo run -p vox-interfaces-cli -- --help
```

## Style

- Automatic formatting only: `cargo fmt`. Never hand-align code.
- Public items, crates, modules, structs, and functions carry doc comments.
- Inline comments explain *why*, not *what*. Max 80 characters per line.
- No trailing whitespace, no tabs, no emotional or vague comments.
- Tests are deterministic and isolated: no wall-clock, no network, no shared
  global state. Mock at the process boundary (for example a local HTTP server).
- Coverage must not drop more than 5% per PR; critical paths target 100%.

## Commit messages

Conventional commits. Subject imperative, 72 characters or fewer; body explains
WHY, WHAT, IMPACT; footer references the issue.

```text
feat(provider): add streaming SSE parser for chat completions

The OpenAI-compatible client needs incremental token deltas for print mode.
WHY: users need output before the full response completes. WHAT: incremental
Server-Sent-Events decoder with unit tests over chunked fixtures.
IMPACT: enables print mode without waiting for response completion.

Refs #12
```

## Reporting bugs

Open an issue with: environment, exact steps, observed behavior, expected
behavior, and the smallest reproduction you can produce. Do not report
security vulnerabilities publicly; see [security.md](security.md).
