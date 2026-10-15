# ADR-0002: Modular crate layout by architectural layer

- Status: Accepted
- Date: 2026-10-15

## Context

The mandate requires a 7+ layer modular architecture (domain, application,
infrastructure, presentation, interfaces, shared, config) with hard limits:
max 150 lines of code per file, max 5 direct files per folder, max 5
subfolders per folder, no circular dependencies, no generic folder names
(`utils`, `helpers`, `common`, `misc` are forbidden), one folder one domain.

A flat crate list grows unwieldy; a single mega-crate violates the limits.
Crates must group by layer while keeping folders under the file limits.

## Decision

Organize the workspace under `crates/` with three groups:

```text
crates/
  core/        vox-domain  vox-application  vox-config  vox-shared
  platform/    vox-provider/  vox-engine/  vox-runtime/
  product/     vox-interfaces-cli  vox-interfaces-rpc
               vox-presentation-tui  vox-binary? (resolved at first release)
```

- Layer names map to mandate layers: domain, application, shared, config live
  in `core/`; infrastructure lives in `platform/` (providers, engine,
  runtime); presentation and interfaces live in `product/`.
- Crate names are `vox-<layer>-<domain>`, kebab-case, descriptive.
- Dependencies point inward: `product/` may depend on `platform/` and `core/`;
  `platform/` may depend on `core/`; `core/` depends only on itself.
- A workspace dependency-cruiser equivalent (`cargo-deny` plus a hand-kept
  graph in this ADR) is checked in CI when new crates land.

## Alternatives

1. One crate per package of the reference repos.
   Rejected: mirrors foreign structure, spreads small crates too thin.
2. Layers as module folders inside one crate.
   Rejected: cannot enforce compile-time isolation; the mandate forbids god
   crates and layer skipping.
3. Grouped-by-layer crates (chosen).
   Accepted: compile-time isolation, natural home for the file limits, and a
   dependency direction that is checkable.

## Consequences

- Every new capability lands as a crate in its layer group, so the layout
  scales without reshuffles.
- The initial workspace ships only `core/` plus the first infrastructure and
  interface crates; later epics add `vox-engine-edit`, `vox-runtime-lsp`,
  subagent support, and the TUI.
- Cross-crate APIs are public and documented; crates do not reach into each
  other's private modules.

## Justification

Grouping by layer gives compile-time boundaries matching the mandate's layer
rules while keeping each folder small. The grouping rule `product -> platform
-> core` makes cycles structurally impossible.
