# ADR-0006: Session storage format

- Status: Accepted
- Date: 2026-10-15

## Context

The agent keeps durable sessions: conversations, tool results, edits, and
subagent transcripts. Sessions must survive crashes, be inspectable by the
user, and append without rewriting history. References use JSONL transcripts
plus a durable conversation store.

## Decision

On-disk session format is newline-delimited JSON (JSONL) under the platform
data directory:

```text
<data-dir>/sessions/<session-id>.jsonl
```

One event per line, each event carrying a monotonically increasing sequence
number, a timestamp, and a typed payload. Event types: `message`, `tool_call`,
`tool_result`, `edit_applied`, `undo`, `session_meta`. Writes are append-only
with `O_APPEND`; readers tolerate a trailing partial line (crash safety).

In-memory state (the agent loop's working set) is rebuilt by replaying the
JSONL on session resume. Compaction produces a `compacted` event summarizing
older turns; the raw events remain in the file.

Secrets never enter the session file: tool results that may contain
credentials are redacted before appending, and the redaction rules live in
`vox-shared` with tests.

## Alternatives

1. SQLite.
   Rejected for v1: binary blobs are not user-inspectable and complicate
   partial-write recovery; revisit at scale.
2. One JSON document per session.
   Rejected: every append rewrites the whole file; a crash mid-write destroys
   the session.
3. Append-only JSONL (chosen): inspectable, crash-safe, trivially replayable.

## Consequences

- Session files are git-diff friendly; users can review and share them.
- Replay logic is a pure function over lines, unit-testable with fixtures.
- Subagent transcripts reuse the same format in a sibling directory, linked by
  a `parent` field in `session_meta`.

## Justification

Append-only JSONL gives crash safety and transparency with no dependencies,
matching the durable runtime the mandate requires, while keeping the user's
data readable and auditable without special tooling.
