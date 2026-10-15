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

The model has six tools: `read`, `edit`, `write` (hashline patching,
ADR-0005), the two search tools `grep` (literal substring) and
`glob` (path patterns) from ADR-0010, and `bash` (ADR-0011): one
argv command with a mandatory timeout and capped output. Search
output is capped and the cap is announced, so a run over a large
tree stays bounded.

## 4. Try the interactive UI

```sh
cargo run -p vox-interfaces-cli
```

`vox` with no `-p` opens the terminal UI described in
[ADR-0012](../adr/presentation/0012-interactive-tui.md) and in the
README. Type a prompt and press Enter; `/help` inside the surface lists
the keys and commands. The interactive session writes one durable
session file, so the same store that print mode uses now backs the
conversation.

## 5. Find your first task
>>>>>>> a054f40 (feat(presentation): interactive terminal UI as the default entry point)

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

Print mode and the interactive TUI are implemented end to end against
OpenAI-compatible providers. The tool harness (read, edit, write over
the hashline edit engine), the bounded and durable agent loop, and the
session store are in place. The search and execution tools
(grep, glob, bash), language-server integration, subagents, and the
editor RPC surface are tracked in the issue roadmap.

| Path | Contents |
| --- | --- |
| `crates/product/vox-presentation-tui/` | the interactive surface: state, rendering, terminal shell |
| `crates/platform/vox-agent/` | the bounded, durable tool-use loop |

The full layer map lives in [docs/index.md](../index.md).
