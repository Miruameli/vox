# Changelog

All notable changes to this project are documented in this file. The format is
based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Fixed

- Onboarding documented the deleted REPL (`vox>` line mode with
  `/session`, `/sessions`, `/resume`, none of which exist in the
  TUI) and carried a committed merge-conflict marker; README
  showed the same stale example. Both now teach the actual TUI
  entry point (ADR-0012) and the documented session-flag contract
  ([#36](https://github.com/Miruameli/vox/issues/36)).

- The TUI seeded the stored conversation for plain `--session <id>`
  as well as `--resume <id>`, silently continuing stale turns when
  only the file to write was named; only `--resume` now continues a
  conversation. Print mode ignored the session flags entirely, which
  is indistinguishable from a lost session; it now fails with an
  actionable error
  ([#34](https://github.com/Miruameli/vox/issues/34)).

- `bash` returned a finished command's answer only at the end of
  its whole timeout budget when a background process held the
  pipes open (`sh -c "echo ready; sleep 30 &"` with a 30s budget
  returned at t=30s although the answer was ready at t=0)
  ([#33](https://github.com/Miruameli/vox/issues/33)). Once the
  child is reaped (or killed), pipe collection is bounded by a
  short post-exit grace: the command's own output was written
  before it exited, and a daemon it left behind is not the
  command. Streams a descendant still holds are reported as
  incomplete, unchanged.

- `grep` returned matches in lexicographic order, so
  `big.rs:100:` outranked `big.rs:11:` and the 100-line cap showed a
  lexicographic page instead of the first matches
  ([#27](https://github.com/Miruameli/vox/issues/27)). The tool now
  sorts the file list it walks (walk order is unspecified by
  contract) and relies on per-file line order from enumeration;
  ordering is pinned by tests, including under the cap. `glob`
  rejects an empty pattern, matching `grep`'s existing validation.

### Added

- `bash` tool (platform layer, ADR-0011): runs one argv array with
  the project root as working directory, no shell involved, so
  there is no quoting or string interpolation to get wrong. A
  timeout is mandatory (default 30s, hard ceiling 60s), output is
  capped per stream with the cut announced, and a non-zero exit is
  a result the model reads rather than a tool failure. A command
  the agent cannot start or that overruns is reported as a failure
  value the loop can act on.
- `vox-presentation-tui` (product layer, ADR-0012): the interactive
  terminal UI, now the default entry point of `vox`. A scrolling
  transcript with labelled turns, tool activity shown as it runs with
  an expandable, length-marked preview of what each tool returned, a
  prompt line with recall, slash commands (`/quit`, `/cancel`,
  `/clear`, `/tools`, `/expand`, `/help`), mouse wheel and page
  scrolling that never moves what the user is reading, and a status
  line that always says what the agent is doing. Rendering is
  `ratatui` 0.30.2 over `crossterm` 0.29.0, styled with modifiers only
  so the surface follows the user's terminal theme; the state machine
  is pure, so keys, events, wrapping, and frames are tested without a
  terminal. `vox-agent` gained `Agent::continue_conversation`, which
  keeps a conversation across prompts, and its event sink is now
  `Send` so a host can run the agent on a task of its own.
- `vox-agent` (platform layer, ADR-0009): the bounded, durable tool-use
  loop. One run is one prompt: request with tools, execute each call
  through the tool harness, append every message/call/result to the
  session before the next request. Bounds cap turns (25) and
  consecutive tool failures (3); a shared flag cancels between actions;
  resume rebuilds the conversation from the file. The provider contract
  gained `CompletionRequest.tools` and `ModelProvider.complete()`
  returning `AssistantTurn`; the OpenAI adapter maps tool calls to the
  wire `tool_calls` array and tool results to `tool` role messages
  (new `wire/` module split), verified by recorded-vector tests.
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
