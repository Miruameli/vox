# Onboarding

New contributor? This page gets you from clone to a passing build in under
five minutes.

## 1. Clone and build

```sh
git clone https://github.com/Miruameli/vox.git
cd vox
cargo build --workspace
```

## 2. Run the checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 3. Try the CLI

```sh
cargo run -p vox-interfaces-cli -- --help
cargo run -p vox-interfaces-cli -- config
# interactive mode is the default: one prompt per line, slash commands:
OPENAI_API_KEY=... cargo run -p vox-interfaces-cli --
#   vox> list the files
#   vox> /session          (show this session id)
#   vox> /sessions         (list stored sessions)
#   vox> /resume vox-1730000000
#   vox> /quit
# print mode against any OpenAI-compatible endpoint:
OPENAI_API_KEY=... cargo run -p vox-interfaces-cli -- -p "list the files"
```

Interactive mode runs the real agent loop: every prompt executes tool
calls, streams events (`-- turn 1 --`, `[tool] read ok`), and appends
each event to a durable JSONL session under the data dir. `--session
<ID>` picks the file, `--resume <ID>` continues an existing session
with its full conversation, and Ctrl-C cancels a run at the next safe
point without ending the session.

## 4. Find your first task

- Read [docs/index.md](../index.md) for the architecture map.
- Browse issues labelled `good first issue`.
- Read [contributing.md](../governance/contributing.md) before opening a PR.

## Where things live

| Path | Contents |
| --- | --- |
| `crates/core/` | domain, application-facing config, shared primitives |
| `crates/platform/` | infrastructure: providers, engines, runtimes |
| `crates/product/` | interfaces and presentation |
| `docs/adr/foundation/` | architecture decision records (core) |
| `docs/adr/agent-runtime/` | ADRs for the agent runtime |
| `docs/adr/operations/` | ADRs for configuration, trust, and quality gates |
| `docs/runbook/` | development and release procedures |

## Current state

Print mode is implemented end to end against OpenAI-compatible providers.
The interactive TUI, tool harness, edit engine, and language-server
integration are tracked in the issue roadmap.
