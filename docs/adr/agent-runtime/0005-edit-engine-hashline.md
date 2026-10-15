# ADR-0005: Edit engine with content-hash anchored patches

- Status: Accepted
- Date: 2026-10-15

## Context

The most failure-prone agent tool is file editing. Model-authored patches fail
on whitespace drift, retyping of unchanged lines, and stale reads of files that
changed between turns. Reference implementations show two fixes: hashline
patches (the model addresses lines by content hash, so stale anchors are
rejected before corruption) and AST-scoped edits for mechanical rewrites.

## Decision

The edit engine (`platform/vox-engine/vox-engine-edit`) applies patches in one
of two modes, chosen by the model or configuration:

1. **Hashline patches (default).** Each target line is addressed by its
   content hash. The engine verifies anchors against the current file state
   before applying; any mismatch rejects the whole patch. The model never
   retypes unchanged context, cutting output tokens and whitespace disputes.
2. **Structured whole-file writes** for new files or full rewrites, applied
   atomically.

All edits are atomic: write to a temporary file in the same directory, then
rename. A failed edit leaves the original byte-for-byte intact. An undo
journal records the pre-image of every applied edit for rollback.

AST-scoped rewrites (tree-sitter based, `vox-engine-ast`) come later as a
separate tool behind the same review card; they are out of scope for the
first milestone.

## Alternatives

1. Plain string-replace edits.
   Rejected: the dominant failure mode in references; silent corruption when
   the anchor no longer matches.
2. Full unified-diff parser.
   Rejected: diff parsing is error-prone for models and hard to repair; diffs
   also carry no integrity guarantee.
3. Hashline patches plus atomic writes (chosen): integrity is enforced by the
   anchor check, and atomic apply makes failure safe by construction.

## Consequences

- Models must address edits by hash, which requires the read tool to return
  hashed views; this is part of the read-tool design in the first milestone.
- Deterministic tests: patch fixtures with stale anchors must be rejected;
  patch fixtures with valid anchors must apply exactly.
- The undo journal is append-only JSONL next to the session store (ADR-0006).

## Justification

Content-hash anchoring converts a class of silent corruption into a loud,
testable rejection, and atomic writes make the remaining failure modes safe.
Both properties are required by the mandate's security-first rule for tools
that mutate the user's filesystem.
