# ADR-0008: Quality gates and CI pipeline

- Status: Accepted
- Date: 2026-10-15

## Context

The mandate defines merge gates: lint 0 errors, tests green, no known
vulnerabilities, coverage drop at most 5%, performance regression at most 10%,
file and folder limits, no circular dependencies, permissive licenses only.
These gates must be enforced by automation, not memory.

## Decision

One CI pipeline on GitHub Actions, `ubuntu-latest` with the pinned stable
toolchain, no path filters (path-filtered workflows strand required checks):

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo audit --deny warnings` (advisory database)
5. `cargo deny check` (licenses, bans, duplicate versions)
6. Structure gate: a script that fails when any file exceeds 150 lines of
   code, any folder exceeds 5 direct files or 5 subfolders, or new forbidden
   folder names appear (`utils`, `helpers`, `common`, `misc`).

Coverage (`cargo llvm-cov`) and performance baselines are added when the first
end-user surface (print mode, then interactive TUI) lands; the gate thresholds
are set then, with the drop-at-most-5% rule from the start.

Dependabot runs weekly for Cargo updates; security-relevant updates open
immediate PRs. Secrets scanning (gitleaks) runs on push and PR.

## Alternatives

1. Path-filtered workflows.
   Rejected: breaks required status checks for docs-only PRs; recorded as a
   hard lesson from operating this repository.
2. Local pre-commit hooks only.
   Rejected: bypassable; the mandate requires CI to be the authority.
3. Single unfiltered pipeline with structure gate (chosen): every PR faces
   every gate, which is the point.

## Consequences

- PRs cannot merge with red CI; the merge step is `gh pr merge --squash` after
  `gh pr checks --watch --required` passes.
- The structure gate is a script under `scripts/` with its own tests; changes
  to limits are ADR-worthy.
- A new crate requires updates to the workspace members list, the layer map in
  the contributing guide, and the dependency direction check.

## Justification

Gates that run on every PR convert the mandate's rules into mechanical
checks. Unfiltered workflows with required checks make bypass impossible
without disabling branch protection, which only the owner can do.
