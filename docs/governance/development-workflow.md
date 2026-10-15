# Development workflow

The workflow is strict. The rules below are enforced in review and CI.

## Default workflow

```text
issue -> branch -> code -> test -> security scan -> self-review
      -> PR -> CI pass -> review -> merge -> update -> (release)
```

- Every change starts from an issue. Maintainers open their own issues.
- Never push directly to `main`. The branch is protected.
- Every PR references its issue. PRs without an issue are rejected.
- CI must be green and security scans must pass before merge.
- Delete the branch after merge.

## Branch prefixes

| Prefix | Use |
| --- | --- |
| `feat/` | new features |
| `fix/` | bug fixes |
| `hotfix/` | urgent production fixes |
| `refactor/` | behavior-preserving restructuring |
| `docs/` | documentation only |
| `chore/` | tooling, dependencies, CI |

## Commit rules

- Conventional commit subject, imperative, 72 characters or fewer.
- Body: WHY, WHAT, IMPACT. Footer: issue reference.
- No emojis, no jokes, no vague messages such as "fix stuff".

## Pull request checklist

A PR is mergeable only when all of these hold:

- [ ] Code works; the changed path was exercised by a run or a test
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes; no flaky tests introduced
- [ ] No hardcoded secrets; secret scan passes
- [ ] No new vulnerability from dependencies; audit passes
- [ ] Self-review completed; diff understood line by line
- [ ] Issue referenced in the PR body and commit footer
- [ ] Every new or changed file is 150 lines of code or fewer
- [ ] Every changed folder has 5 or fewer direct files
- [ ] No circular dependency between crates
- [ ] Coverage did not drop more than 5%
- [ ] Documentation updated in the same PR
- [ ] Rollback path described in the PR body for risky changes

## Issue rules

- Issues are written before work. One issue covers one outcome.
- Labels: `bug`, `enhancement`, `documentation`, `architecture`, `security`,
  `ci`, `dependencies`. Use only existing labels.
- Update the issue when scope changes; close it from the PR that resolves it.

## Review rules

- Review the diff against the checklist above, not against personal taste.
- Blocking review comments cite a rule, a defect, or a risk.
- Non-blocking comments are prefixed with `nit:`.

## Emergency changes

For critical security or outage fixes: open the issue, branch `hotfix/`, ship
the smallest safe change, and follow up with tests and documentation in the
same PR or an immediately referenced follow-up issue. The security SLA in
[security.md](security.md) still applies.
