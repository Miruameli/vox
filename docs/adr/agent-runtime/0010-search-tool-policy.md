# ADR-0010: Search tools and their output policy

- Status: accepted
- Date: 2026-10-16
- Layer: platform (`vox-engine-tools`)

## Context

The tool harness shipped read, edit, and write: the model could open
a known path, patch it by content hash, and create it. Every real
coding task starts earlier than that - locating the symbol, finding
every call site, listing the files a module owns - and doing it with
`read` alone costs one round trip per file, which is both slow and
context-expensive.

Adding search raises decisions the read tool never faced: output
size is unbounded (a `target` directory holds thousands of matching
lines), a pattern language implies a dependency (a regex engine is a
new crate and a new attack surface), and a walk must decide what it
refuses to enter.

## Decision

Two tools join the harness:

- `grep`: literal substring search. No regex. The pattern goes
  through `str::contains`, so there is nothing to compile, nothing
  to backslash, and no ReDoS. A model that needs alternation issues
  two calls; a real regex engine can arrive later as its own ADR
  with the dependency cost spelled out.
- `glob`: path patterns with `*` and `?` inside a segment and `**`
  spanning directories, implemented in-seat (about 30 lines).

Both share `walk::files`, which skips dotfiles and the directories
`.git`, `node_modules`, `target`: dependency and build output dwarfs
the source tree and would spend the output cap on noise.

Output policy, shared by both:

- A hard cap - 100 matching lines for grep, 200 paths for glob -
  because search cannot reject large requests the way `read` does:
  the model cannot know the match count before asking. Truncation is
  announced in the result itself (`... N more matches`), never
  silent. The model-facing principle from ADR-0008 holds: bounded
  output, visible bound.
- Results are sorted before the cap, so the same query always
  returns the same page.
- Both resolve paths through `vox_engine_edit::resolve_within`, so
  absolute paths and `..` escapes fail as tool results, exactly like
  `edit`.

## Alternatives considered

- Regex grep via the `regex` crate: rejected for now. A new
  dependency for the platform layer needs its own ADR under the
  tooling rules (license, maintenance, transitive count), and the
  literal tool covers the locating need; the dependency-free
  version ships today.
- Ignore-file semantics: rejected. No parser in the repo, and the
  skip list covers the expensive offenders. Deferred with the
  filesystem-watcher ADR.
- Streaming matches without a cap: rejected. An unbounded tool
  result is an unbounded context bill and a prompt-injection
  surface; the cap plus marker is the cheaper contract.

## Consequences

- Five tools are registered with providers; the definitions pin in
  `vox-agent` was updated in the same change.
- `src/` holds `harness.rs`, `lib.rs`, `walk.rs`, and a `tools/`
  directory with one file per tool, keeping the folder at five
  direct files as the structure gate requires.
- Adding a regex engine later is additive: the `grep` tool keeps
  its literal mode or gains a `mode` argument under a new ADR.
- `target/` is skipped by name only; a project that builds into
  `build/` gets noise until an ignore-file rule lands.
