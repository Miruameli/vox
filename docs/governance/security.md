# Security policy

## Supported versions

Security fixes target the latest released minor line only.

## Reporting a vulnerability

Do not open a public issue. Use GitHub private vulnerability reporting on
<https://github.com/Miruameli/vox/security/advisories/new>, or contact the
maintainer privately. Include reproduction steps, impact assessment, and
affected versions.

Response targets:

| Severity | Response | Fix target |
| --- | --- | --- |
| Critical | 24 hours | 7 days |
| High | 72 hours | 14 days |
| Medium | 1 week | 30 days |
| Low | 1 month | next minor release |

## Secrets

- Credentials never enter the repository: no hardcoding, no `.env` commits.
- Provider API keys are read from the environment or the OS credential store.
- The agent never sends secrets to third parties. No key proxy exists.

## Supply chain

- Dependencies are pinned by `Cargo.lock`; CI runs `cargo audit`.
- Dependabot opens pull requests for security-relevant updates.
- New dependencies require: permissive license (MIT/Apache-2.0/BSD/ISC),
  maintained upstream (commit within 6 months, issue response under a month),
  no critical or high vulnerabilities, and justified necessity in the PR.

## Hard rules

- Validate all external input by type, length, format, and range.
- Never log secrets or personal data.
- Never disable TLS verification.
- Prefer standard crypto only: AES-256-GCM, RSA-2048+, ECDSA P-256+, SHA-256+.
- No `eval`, no shell string interpolation of untrusted input.
- Errors returned to users are generic; details go to internal logs.

## Agent safety (product scope)

The agent executes commands with the permissions of the invoking user. The
trust model, approval prompts for destructive tools, and undo/rollback design
are specified in [ADR-0007](../adr/operations/0007-configuration-and-trust.md).
