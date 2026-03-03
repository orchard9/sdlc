# Security Audit: dev-driver tool

## Scope

`.sdlc/tools/dev-driver/tool.ts` — a stock tool that reads project state and
spawns a Claude agent process. Reviewed for: command injection, path traversal,
lock file integrity, spawn safety, input validation.

---

## Findings

### 🔴 HIGH — None

### 🟡 MEDIUM — Shell injection risk in spawn command

**Location:** `spawnClaude()` — `['--print', `/sdlc-next ${feature.feature}`]`

The `feature.feature` value comes from `sdlc next --json` output (the `feature` field).
If a slug contains shell metacharacters (spaces, semicolons, backticks), they would be
passed as part of the `--print` argument string to claude.

**Analysis:** The sdlc CLI enforces slug format as `[a-z0-9-]+` at creation time. Slugs
cannot contain spaces or shell metacharacters by construction. The value passes through
`sdlc next --json` → JSON parse → TypeScript string before reaching spawn. No shell
expansion occurs since `spawn()` (not `exec()`) is used — arguments are passed directly
to the OS without shell interpretation.

**Resolution:** No code change needed. The spawn call uses array form (not shell string),
which bypasses shell interpretation entirely. The slug origin (sdlc JSON output) is
controlled. Documenting for awareness.

### 🟡 MEDIUM — Lock file race condition (TOCTOU)

**Location:** `readLock()` / `writeLock()` — check-then-write

If two dev-driver instances run simultaneously (e.g. manual + scheduled trigger at same
millisecond), both might read "no lock" before either writes one. Both would then dispatch.

**Analysis:** The orchestrator's action dispatch is single-threaded (one tick loop). The
only scenario for concurrent runs is external invocation + orchestrator tick at the exact
same moment. The 2h TTL means a double-dispatch results in at most 2 agent runs targeting
the same feature, each running `/sdlc-next` — the state machine handles this correctly
(second run finds nothing to do or runs the same step redundantly).

**Resolution:** Accept. The risk is low (sub-millisecond collision window), the consequence
is a redundant agent run (not data corruption), and adding OS-level file locks would
significantly complicate the tool. The flight lock is defense-in-depth, not a hard mutex.

### ✅ LOW — Path traversal in lock file

`lockPath(root)` constructs `.sdlc/.dev-driver.lock` using `join(root, '.sdlc', ...)`.
`root` comes from `process.env.SDLC_ROOT ?? process.cwd()`. These are both trusted
sources (process environment, working directory). No user-controlled path components.

### ✅ LOW — Quality check output parsing

`runQualityCheck()` calls `JSON.parse()` on output from a local Node.js process.
Wrapped in try/catch. If parsing fails, returns `{ failed: 0, failedNames: [] }` —
a safe default that allows the tool to continue rather than crash.

### ✅ LOW — `sdlc run list` fallback

`hasActiveRuns()` is fully wrapped in try/catch. If the command doesn't exist or
returns unexpected output, the function returns `false` (safe: allows dispatch to proceed).
This is the correct behavior for graceful degradation.

### ✅ LOW — No network access

The tool makes no network requests. All I/O is: local file reads (lock, tasks.md), child
process execution (sdlc commands, quality-check, claude spawn), and JSON parsing of their
outputs. No external service dependencies.

### ✅ LOW — No secrets handling

The tool does not read, write, or transmit credentials, tokens, or sensitive configuration.
`SDLC_ROOT` is passed to spawned processes but this is a filesystem path.

---

## Verdict: APPROVED with notes

No high-severity findings. Two medium findings are both accepted with documented rationale:
1. Spawn injection: mitigated by slug format constraints and `spawn()` array form (no shell)
2. TOCTOU race: accepted given narrow window and benign consequence (redundant step, no corruption)

The tool is production-appropriate for its intended use as a local development orchestrator.
