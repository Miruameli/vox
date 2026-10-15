# ADR-0012: Interactive terminal UI

- Status: Accepted
- Date: 2026-10-15
- Deciders: Miruameli (project owner)

## Context

The runtime is complete and the product is not reachable. `vox-agent`
runs a bounded, durable tool-use loop and emits `LoopEvent`s to a sink
(ADR-0009), `vox-session` persists every event, and
`vox-engine-tools` dispatches read/edit/write. What is missing is the
presentation layer that ADR-0002 reserved: `crates/product/` holds only
the CLI interface, and `vox` without `-p` exits with code 2. An agent
that can only be reached one prompt per process is not a coding
terminal.

Four decisions had to be made before code existed: where the surface
lives, what draws it, how a blocking render loop and an async model
loop share one terminal, and how a conversation survives more than one
prompt. The reference products (`earendil-works/pi`, `can1357/oh-my-pi`)
set the bar for what a coding-agent terminal looks like: a scrolling
transcript with labelled turns, tool activity shown as it happens, a
prompt line with recall, slash commands, and a status line that always
answers "what is the agent doing right now".

## Decision

**1. New crate `vox-presentation-tui` in `product/`.** It depends on
`vox-agent`, `vox-domain`, and `vox-shared`; it does not depend on
`vox-interfaces-cli`. The CLI stays the composition root: it resolves
configuration, reads the credential, builds the provider, the harness,
and the session, and hands the assembled `Agent` to the TUI. The
presentation layer never reads the environment or the config file, so
the same surface can be embedded in another host.

**2. `ratatui` 0.30.2 with `crossterm` 0.29.0, pinned exact.**
Alternatives considered:

- Draw directly with `crossterm` escape sequences. Rejected: buffer
  management, clipping, wrapping, and layout become hand-written code
  inside the presentation crate, which is exactly the kind of
  unrestricted growth the file caps exist to prevent.
- `termion`. Rejected: Unix-only, and the product must run on Windows.
- `tui-realm`. Rejected: a component framework on top of a renderer we
  do not need yet; added abstraction with no present consumer.

The pair was verified on the pinned toolchain (rustc 1.98.1, workspace
`rust-version = 1.98`): `ratatui` 0.30.2 declares MSRV 1.88 and
compiles here, so CI's `stable` cannot drift into a build the workspace
cannot support. `crossterm` 0.28.0 is yanked, which settles the
0.28-versus-0.29 choice. Styling uses modifiers (`BOLD`, `DIM`,
`REVERSED`) and never hardcoded colours, so the surface follows the
user's terminal theme instead of fighting it.

**3. One worker task owns the agent; the render loop never awaits the
model.** A prompt is sent over an mpsc channel to a tokio task that owns
the `Agent` and runs it with a sink that forwards every `LoopEvent`
back to the render loop. Cancellation reuses the loop's existing shared
`AtomicBool` (`Agent::cancel_token`), which the agent already checks
between turns and between tool calls. The render loop blocks only on
`crossterm` event polling with a timeout, then drains the channel, so
the terminal stays responsive while a turn runs.

**4. Multi-turn conversation by seeding, not by rebuilding.**
`Agent::run` starts from an empty conversation by design, so a UI that
called it per prompt would lose context after every turn. The agent
gains `continue_conversation(&mut history, prompt, sink)`: the caller
owns the message list, and `run` becomes the empty-history case. The
loop itself moves unchanged into a private `drive` helper, so the
durability and bounds of ADR-0009 are untouched. The alternative -
rebuilding the `Agent` per prompt - was rejected because the durable
session would restart per turn, and the alternative - passing messages
into `run` - was rejected because it would change the signature every
existing caller uses.

**5. The event sink is `Send`, and tool events carry bounded previews.**
A sink that a caller cannot hand to a spawned task is a sink that no
interactive host can use, so `FnMut(LoopEvent) + Send` is now the
signature. `LoopEvent::ToolStarted` and `ToolFinished` gained a
`detail` field: the tool arguments and a preview of its output, capped
by `vox_agent::TOOL_OUTPUT_PREVIEW` and marked with how much was left
out. An unbounded payload crossing the bridge was the alternative, and
it was rejected: a single read returns up to two thousand lines, and
the complete output already lives in the session file.

## Consequences

- Layout stays inside the file caps: `src/app/` holds the state machine
  (transcript, input, keys, commands, events), `src/input/` the prompt
  line, `src/render/` wrapping and drawing (four files), and the
  terminal shell, the worker, and the event loop are three further
  files. No folder exceeds five direct files, no file exceeds 150
  lines.
- The state machine is a pure reducer over `KeyEvent` and `AppMsg`, so
  it is testable without a tty; rendering is tested through a
  `TestBackend`. Only the terminal shell needs a real terminal, and a
  smoke run against a local mock provider proves that path.
- `vox-agent` gains one public method and one `Send` bound on the sink.
  `run` keeps its contract, so the existing loop, resume, and
  cancellation tests are unchanged; `vox-agent/src/bounds.rs` and
  `events.rs` merged into `contract.rs` and the loop moved to
  `run_loop.rs`, which is a file-layout change with no public-API
  change.
- The TUI is the product's second surface and the first interactive
  one; the RPC surface (ADR-0002, `vox-interface-rpc`) will consume the
  same agent and session crates from a different composition root.

## Alternatives

1. Render inside the CLI crate. Rejected: `vox-interfaces-cli` would
   grow a presentation tree it cannot be reused from, and the
   interfaces layer would skip a layer.
2. Run the agent inline in the render loop. Rejected: the render loop
   would block on network I/O, freezing input and the cancel key.
3. A plugin/widget framework for the UI. Rejected: no second consumer
   exists; build the need when it appears.
4. Colour themes in the first release. Rejected: modifiers already
   adapt to the user's terminal, and a token system without a second
   consumer is abstraction ahead of need. The styles live in one module
   (`render/styles.rs`) so a theme can land there without touching the
   state machine.

## Follow-up

Tool execution is not gated: a tool call runs as soon as the loop
requests it. An approval step - ask before a write, approve or deny,
never approve on a plain Enter - belongs in the loop of ADR-0009, not
in the presentation layer, and is tracked as its own issue.

## Justification

A coding terminal agent must be usable interactively, and usability
here means a responsive surface over a bounded, durable loop. Putting
the surface in its own presentation crate keeps the composition root in
the interfaces layer and keeps the render loop free of model latency,
which is the only way both properties hold at once.
