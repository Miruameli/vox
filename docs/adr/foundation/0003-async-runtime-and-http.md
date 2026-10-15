# ADR-0003: Async runtime and HTTP stack

- Status: Accepted
- Date: 2026-10-15

## Context

The agent is I/O heavy: streaming model responses, PTY sessions, filesystem
walks, language servers over stdio. A single-threaded blocking design cannot
keep a responsive terminal UI during long operations; a thread-per-connection
design does not scale.

## Decision

Use `tokio` (current stable, full feature set) as the async runtime and
`reqwest` (rustls, no OpenSSL) for HTTP. Streaming is consumed as a byte
stream and decoded by crate-local parsers (SSE) so the decoder is unit
testable without a server. Blocking work that cannot be async (PTY control,
some FFI) runs on `tokio::task::spawn_blocking`.

## Alternatives

1. `async-std`.
   Rejected: smaller ecosystem, weaker task instrumentation.
2. Blocking threads with channels.
   Rejected: backpressure and cancellation become hand-rolled.
3. tokio + reqwest (chosen): the ecosystem default, mature tooling, and the
   stack used by both reference projects' Rust crates.

## Consequences

- Every provider client is async and streams incrementally; tests target pure
  parser functions plus a local mock server for the transport.
- `rustls` avoids OpenSSL build concerns across release platforms.
- Cancellation uses `tokio_util::sync::CancellationToken`, giving deterministic
  shutdown for interactive abort.

## Justification

tokio and reqwest are the maintained, broadly audited defaults for this
workload. Isolating SSE parsing as a pure layer keeps the network-adjacent
logic deterministically testable, which the mandate requires.
