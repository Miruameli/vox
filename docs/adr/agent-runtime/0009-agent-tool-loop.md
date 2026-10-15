# ADR-0009: Tool-use contract and the agent runtime loop

- Status: Accepted
- Date: 2026-10-15

## Context

The first milestone shipped the pieces of an agent without the agent:
a layered config, an OpenAI-compatible provider streaming text, the
hashline edit engine, tools that speak `ToolCall`/`ToolResult`, and a
durable session store. Two gaps remained.

1. The provider contract was text-only: `StreamEvent` carried
   `Started`/`Delta`/`Completed` and nothing else. Tool calls could not
   cross the boundary, so no orchestration was possible.
2. Nothing composed provider, tools, and session into a loop, so the
   product could complete a prompt but not act.

The loop is where the mandate's hard requirements meet: agents must be
bounded (no runaway), durable (a crash loses nothing), and honest
(no silent truncation of tool content).

## Decision

**Contract extension (domain).** `CompletionRequest` gains
`tools: Option<Vec<ToolDefinition>>`. `ModelProvider` gains
`complete()`, a non-streaming call returning `AssistantTurn`
(`message`, `finish_reason`, `usage`). Streaming text stays as-is for
print mode; the loop uses `complete()` because tool calls need full
assembly, and a partial tool call is never executable.

**The loop (`platform/vox-agent`).** One run is one prompt:

1. Append the user message to the session.
2. Request a completion with the conversation and the tool
   definitions.
3. Append the assistant message; emit its text.
4. If the finish reason is not `tool_calls` (or no calls arrived),
   stop with `ModelStopped`.
5. Otherwise execute each call through the tool harness, appending
   `tool_call` before and `tool_result` after execution, and continue
   with the results in the conversation.

**Bounds.** `Limits` caps turns (default 25) and consecutive tool
failures (default 3); the run stops with `TurnLimit` or
`ToolErrorLimit`. A shared `AtomicBool` cancellation flag is checked
between turns and between tool calls; the run stops with `Cancelled`.

**Durability.** Every message, tool call, and tool result is appended
and synced before the next request. Resume replays the file; the
assistant `message` event already carries tool calls as content
blocks, so the separate `tool_call` event is the audit record and adds
no duplicate turn.

**UI-agnostic progress.** The loop emits `LoopEvent`s to a callback:
turn started, text, tool started/finished, and the finish reason.
The terminal UI will be one sink among several.

## Alternatives

1. Streaming tool calls as delta events.
   Rejected: partial arguments are not executable; assembly logic
   would duplicate in every consumer.
2. An event-sourced runtime that replays the whole session per turn.
   Rejected: O(n^2) over a long session; the in-memory conversation
   plus append-only file gives the same durability without the cost.
3. Unbounded loop with a watchdog.
   Rejected: the watchdog fires after the damage; explicit bounds make
   the contract testable.

## Consequences

- The OpenAI wire adapter maps `ToolCall` blocks to the `tool_calls`
  array and tool results to `tool` role messages; the message mapper
  grew a `wire/` module split to stay under the file-size gate.
- `complete()` is a required trait method: a provider that cannot do
  tool calling cannot implement `ModelProvider`, so the gap is loud at
  compile time instead of silent at runtime.
- The loop is testable without network: a mock provider scripts turns,
  and the tests assert the session event order, bounds, cancellation,
  and resume reconstruction.

## Justification

Bounded execution, durable state, and loud failure are mandate
requirements, not preferences. The contract extension keeps streaming
text intact for print mode while giving the loop the only shape a tool
call can have: whole, validated, and replayable.
