# ADR-0007: Layered configuration and project trust model

- Status: Accepted
- Date: 2026-10-15

## Context

The agent merges CLI flags, environment variables, project configuration, and
user configuration. It also loads project-controlled inputs (rules, skills,
MCP servers, extensions) that can execute code. The mandate forbids trusting
client-style input and requires explicit, least-privilege behavior.

## Decision

Configuration is layered, later layers override earlier ones:

1. Built-in defaults (compiled in).
2. User config: `<config-dir>/vox/config.toml`.
3. Project config: `./vox.toml` discovered from the working directory upward,
   stopping at the filesystem or repository root.
4. Environment variables prefixed `VOX_`.
5. CLI flags.

All configuration is validated on load: unknown keys are rejected with the
list of valid keys, wrong types are rejected with the key path. No partial
application: a config that fails validation stops the run with a generic
message; details go to the log.

Trust model:

- Project inputs (rules, skills, commands) load only from the project root or
  a directory the user explicitly marks trusted.
- Code-executing extensions are opt-in per source path
  (`--trusted-extension <path>`), never ambient.
- Project MCP servers load only when `mcp.enableProjectConfig` is true, which
  is off by default.
- Credentials come from the environment or OS credential store. Configuration
  files never hold secrets; a secret-shaped value in config is rejected at
  validation time.

Environment variables that affect behavior are documented in one table in the
user documentation and validated by the same schema as flags.

## Alternatives

1. Single config file.
   Rejected: no separation of user and project scope; shared machines break.
2. Environment-first precedence.
   Rejected: surprising in local development; standard practice is flags last.
3. Layered with strict validation (chosen): predictable, auditable, and the
   trust boundaries are explicit rather than ambient.

## Consequences

- The loader crate (`vox-config`) is pure: it takes paths and an environment
  map and returns validated config or a structured error, fully unit testable.
- Adding a setting requires updating the schema, the validation tests, and the
  environment table in the same change.
- The trust flags become part of the CLI contract tested by integration tests.

## Justification

Layered, validated configuration with explicit trust switches is the only
design consistent with the security-first mandate: the agent runs code-shaped
inputs from projects, so consent must be per-source and deniable, and secrets
must have a single safe channel.
