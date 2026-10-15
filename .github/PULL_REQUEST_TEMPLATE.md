## Summary

<!-- One or two sentences: what changed and why. -->

## Issue

Closes #<issue-number>

## Change type

- [ ] feat
- [ ] fix
- [ ] refactor
- [ ] docs
- [ ] chore (CI, dependencies, tooling)

## Checklist

- [ ] Code works; the changed path was exercised by a run or a test
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes; no flaky tests introduced
- [ ] No hardcoded secrets; secret scan passes
- [ ] No new vulnerable dependency; audit passes
- [ ] Self-review completed; diff understood line by line
- [ ] Every new or changed file is 150 lines of code or fewer
- [ ] Every changed folder has 5 or fewer direct files and 5 or fewer subfolders
- [ ] No circular dependency between crates
- [ ] Documentation updated in the same PR (README, docs, ADR as needed)
- [ ] Rollback path described below for risky changes

## Rollback

<!-- Revert this PR, or: <specific steps>. Write "revert the PR" if that suffices. -->
