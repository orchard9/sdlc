# Security Audit: orchestrator-tick-cli

## Scope

`crates/sdlc-cli/src/cmd/orchestrate.rs` and the underlying
`crates/sdlc-core/src/orchestrator/` data layer (action.rs, db.rs).

## Threat Model

This is a local developer CLI daemon running under the invoking user's own
permissions. There is no network listener, no privilege escalation, and no
multi-user or server context. The attack surface is limited to:
1. Malicious input to `sdlc orchestrate add`
2. A tampered `orchestrator.db` file
3. Path traversal via tool names

## Findings

### F1 — Tool name not slug-validated in `add` (low risk, local only)

`run_add` stores the `--tool` value as-is without calling `validate_slug`.
`dispatch` then calls `tool_script(root, &action.tool_name)` which builds
`root/.sdlc/tools/<tool_name>/tool.ts`. A value like `../../etc/passwd` would
resolve outside `.sdlc/tools/`. The `script.exists()` guard prevents execution
of non-existent paths, and the tool is marked `Failed` instead of crashing.

**Impact**: A user who deliberately passes a malformed `--tool` value gets a
`Failed` action — no code execution beyond what the user can already do
directly. Zero privilege escalation. Accepted risk for a local developer tool.
A future hardening task could add `validate_slug(tool_name)?` in `run_add`.

### F2 — Tool input JSON passed via stdin, not as args (no finding — safe)

`serde_json::to_string(&action.tool_input)` is written to the child process's
stdin, not interpolated into a shell command. `build_command` passes `script`
and `mode` as separate `Command::args()` entries. No command injection vector.

### F3 — `mode = "--run"` hardcoded (no finding — safe)

The orchestrator always calls `run_tool(..., "--run", ...)`. The mode is not
derived from user input or DB content.

### F4 — DB at arbitrary `--db` path (accepted)

`--db` allows a custom redb file path. A malicious value can cause reads/writes
to arbitrary redb files on disk. Since the user is already root on their own
machine and this is a local tool, this is accepted. The default path is
gitignored per spec.

### F5 — Tool stdout parse with `unwrap_or(Null)` (no finding)

Invalid JSON from tool stdout becomes `Completed { result: Null }` rather than
crashing the daemon. This is the intended graceful degradation path.

### F6 — Startup recovery (no finding)

`startup_recovery` only transitions `Running → Failed` for actions older than
`2 * tick_rate`. It does not execute any code or read user-supplied values
beyond what was already in the DB.

### F7 — No network exposure (no finding)

The daemon is a pure local process. No sockets opened.

## Summary

| Finding | Severity | Disposition |
|---------|----------|-------------|
| F1: tool name unvalidated in `add` | Low | Accepted — local tool, no privilege escalation |
| F4: `--db` arbitrary path | Negligible | Accepted — local CLI, same user |
| All others | N/A | No findings |

**Verdict: PASS.** No blocking security issues. The feature's attack surface is
limited to the local developer machine with the user's own credentials.
