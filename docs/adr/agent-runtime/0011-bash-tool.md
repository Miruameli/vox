# ADR-0011: The bash tool — running commands from the agent loop

- Status: accepted
- Date: 2026-10-16
- Layer: platform (`vox-engine-tools`)

## Context

The harness could read, write, patch, and search. A coding agent
that cannot run anything cannot build, test, or scaffold: the
common close of a task ("run the tests, fix what breaks") needs a
process. This ADR fixes the contract before the code exists.

## Decisions

### 1. Arguments are an argv array, not a shell string

`command` is `Vec<String>`; the harness spawns it with
`std::process::Command` directly. No `sh -c` wrapper, so there is
no string interpolation into a shell and no quoting code to get
wrong. The model that needs pipes writes a script file with the
write tool and runs it. A shell-string variant is a separate,
later decision.

### 2. Working directory is the project root

Commands run with cwd = the harness root. There is no path
argument to escape: the one sandbox the file tools have
(`resolve_within`) does not apply to a process, and the honest
boundary is the root itself. A future sandbox runtime is a new
ADR.

### 3. Timeout is mandatory and bounded

Default 30 seconds, hard maximum 60 seconds. Timeout is part of
the tool's argument schema with a fixed ceiling — an agent loop
must never be able to hang a session. A timed-out command is
killed and reported as a failure VALUE (see 5).

### 4. Output is hard-capped and the cap is announced

Same policy as ADR-0010: stdout and stderr are each capped
(BASH_OUTPUT_LIMIT bytes) and truncation is stated in the result.
Unbounded output would blow up the session file and the next
request's context.

### 5. A non-zero exit is a result, not an error

The result text carries the exit code, then stdout, then stderr.
The model must read a failing test to fix it. Only failures to
run at all — no such binary, spawn refused — are failure values.
This mirrors the distinction the file tools already make between
actionable failures and infrastructure errors.

### 6. The environment is inherited

Commands inherit the agent's environment: `PATH` is how the
model finds real build tools, and the agent already runs with
the user's credentials. Output may contain project secrets — the
same exposure reading a file already has; the response is the
session file's protection, not a per-tool env scrubber.

## Alternatives considered

1. **Allowlist of commands** — rejected: it blocks real builds
   (`cargo`, `npm`, `git` with subcommands) and adds a second
   trust model on top of the project trust in ADR-0007.
2. **Shell string with `sh -c`** — rejected: shell quoting is a
   recurring source of injection-shaped mistakes; the argv form
   removes the class of problem for free.
3. **Exit code as infrastructure error** — rejected: the model
   acts on failing output; turning it into an error hides the
   signal that matters at exactly the moment the model needs it.

## Consequences

- `bash` joins the five tools in dispatch and provider
  registration; the tool-count pin in `vox-agent` tests moves to
  six.
- The tool lives at `src/tools/shell/bash.rs`: a new subfolder
  under `tools/` keeps every folder inside the 5-direct-file
  gate once the sixth tool arrives.
- Long-running loops stay bounded: the turn cap (ADR-0009) and
  the timeout here compose — worst case per bash call is 60s.
