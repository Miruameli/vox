# Development runbook

How to develop Vox locally. Setup target: under 5 minutes on a machine with
Rust 1.98+.

## Setup

```sh
git clone https://github.com/Miruameli/vox.git
cd vox
cargo build --workspace
```

## Daily loop

```sh
cargo build --workspace                 # compile
cargo test --workspace                  # unit + integration tests
cargo fmt --all --check                 # formatting gate
cargo clippy --workspace --all-targets -- -D warnings   # lint gate
cargo test -p vox-interfaces-cli        # CLI integration tests only
```

## Toolchain versions

Development uses the same toolchain as CI. Check locally:

```sh
rustc --version   # expect 1.98.x line
cargo --version
```

If a future edition requires a newer toolchain, the change includes a CI
update and a note in this runbook.

## Workspace layout

See [ADR-0002](../adr/foundation/0002-modular-crate-layout.md). Crates under
`crates/` are grouped by architectural layer: `core/`, `platform/`, `product/`.

## Testing rules

- Unit tests live next to the code (`#[cfg(test)]` modules or `tests/`).
- Integration tests that need a network boundary use a local mock server; no
  test performs real network calls.
- Tests are deterministic: no wall-clock sleeps, no random seeds without
  fixed values, no dependence on test order.

## Debugging

- `cargo run -p vox-interfaces-cli -- --help` lists CLI flags.
- Set `VOX_LOG=trace` (see [ADR-0007](../adr/operations/0007-configuration-and-trust.md))
  for structured logs on stderr.
- Debug builds keep symbols; release builds do not strip until release.

## CI parity

CI runs exactly the commands listed in the daily loop plus `cargo audit`.
If CI fails on a check that passes locally, update the toolchain first, then
report the pipeline.
