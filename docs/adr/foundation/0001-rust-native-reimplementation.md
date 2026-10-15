# ADR-0001: Rust-native reimplementation of the Pi coding agent

- Status: Accepted
- Date: 2026-10-15
- Deciders: Miruameli (project owner)

## Context

The project reimplements the Pi coding agent as a native Rust product named
Vox. Two references exist: the TypeScript original (`earendil-works/pi`) and
an IDE-wired TypeScript-plus-Rust fork (`can1357/oh-my-pi`). The product target
is a coding terminal agent with IDE-grade features: in-process search and
editing, language-server intelligence, subagents, durable sessions, and
extensibility, but no GUI beyond the terminal.

Constraints from the project mandate: quality over speed, security first,
modular architecture with hard file and folder limits, and a full engineering
process (issues, branches, PRs, CI, ADRs).

## Decision

Reimplement in Rust as a clean-room product. No code is copied from either
reference; both are design inputs only. The workspace is a Cargo workspace
organized by architectural layer (ADR-0002). TypeScript, Node, and Bun are not
used in the product. The agent is terminal-only: interactive TUI, print mode,
and an RPC surface, with editor integration via protocol adapters, never a
GUI.

## Alternatives

1. Fork and rebrand the TypeScript Pi (the `vox-rebuild` experiment).
   Rejected: cannot reach native performance or the single-binary distribution
   the mandate requires; inherits the Node runtime dependency.
2. Port only the hot paths to Rust behind the TypeScript harness (the
   oh-my-pi approach).
   Rejected: dual-runtime complexity for distribution and debugging; the
   mandate forbids needless abstraction, and the N-API bridge is exactly that.
3. Full Rust native rewrite.
   Chosen: one language, one binary, in-process hot paths, deterministic CI.

## Consequences

- Everything shipped by TypeScript references is re-implemented, including the
  provider layer, session store, TUI, tools, and plugin surfaces.
- The Rust ecosystem (tokio, reqwest, tree-sitter, napi-free) covers the hot
  paths: ripgrep via `grep-*` crates, glob via `ignore`/`globset`, diffing,
  SSE parsing, PTY.
- Editor integration lands later through an ACP-style protocol crate
  (`vox-interface-rpc`), matching how references expose editor surfaces.
- The rewrite is tracked as epics in the issue tracker with a roadmap in the
  changelog.

## Justification

The mandate sets quality and security above speed and forbids dual-runtime
complexity. A native binary with in-process hot paths is the only option that
satisfies single-artifact distribution, in-process search and editing, and a
strict supply chain.
