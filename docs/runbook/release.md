# Release runbook

Releases are automated, signed, and reproducible. A release is never source
code only: every release ships binaries for all supported platforms with
checksums and an SBOM.

## Versioning

Semantic Versioning. The version in the workspace `Cargo.toml` is the single
source of truth.

## Pre-release checklist

- [ ] `main` is green: fmt, clippy, tests, audit, coverage gate
- [ ] Every issue in the milestone is closed or moved
- [ ] `docs/changelog.md` has a final `## [x.y.z] - YYYY-MM-DD` section
- [ ] No `TODO` without an issue reference
- [ ] Integration smoke passed: mock-server print mode, config validation

## Release process

1. Open a `release/` branch from `main`; bump the version; update the
   changelog date.
2. Open a PR; wait for CI; merge after review.
3. Tag the merge commit: `git tag -s vX.Y.Z -m "vox vX.Y.Z"`.
4. Push the tag. The release workflow builds release binaries for:
   - `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`
   - `aarch64-unknown-linux-gnu`
   - `x86_64-apple-darwin`, `aarch64-apple-darwin`
   - `x86_64-pc-windows-msvc`
5. The workflow generates `SHA256SUMS` and an SBOM (CycloneDX) and attaches
   them to the GitHub release, then publishes release notes from the
   changelog section.
6. Verify: download one artifact per platform, check the checksum, run
   `vox --version`, record the result on the issue.

## Rollback

- Bad release: unpublish the release, delete the tag, cut `hotfix/` from the
  last good tag.
- Broken dependency: revert the dependency PR, not the consuming code.

## Post-release

- Announce on the repository discussion channel.
- Update the milestone and close the release issue.
