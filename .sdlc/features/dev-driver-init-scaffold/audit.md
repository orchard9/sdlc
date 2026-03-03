# Security Audit: dev-driver scaffolded by sdlc init and sdlc update

## Scope

This audit covers the security posture of the changes made in this feature:
- `crates/sdlc-cli/src/cmd/init/templates.rs` — two new `include_str!` constants
- `crates/sdlc-cli/src/cmd/init/mod.rs` — two new file writes in `write_core_tools()`

The audit does NOT re-audit the dev-driver tool logic itself (audited separately as part of
the `dev-driver-tool` feature).

## Attack surface analysis

### 1. include_str! at compile time

**Finding:** The constants use `include_str!("../../../../../.sdlc/tools/dev-driver/tool.ts")`.

**Risk:** None. `include_str!` is evaluated at compile time by rustc. The included content is
embedded as a literal string constant in the binary. There is no runtime file I/O, no path
traversal at runtime, and no ability for an attacker to influence which file is embedded
post-compilation.

**Verdict:** No issue.

### 2. File writes in write_core_tools()

**Finding:** Two new file writes:
- `io::atomic_write` for `tool.ts` — overwrites on every run
- `io::write_if_missing` for `README.md` — only writes if absent

**Risk (tool.ts overwrite):** If an attacker can influence the binary (replace the `sdlc`
binary with a malicious one), they could embed malicious content in `TOOL_DEV_DRIVER_TS` and
overwrite `tool.ts` on every `sdlc init` / `sdlc update` call. However, this risk is
equivalent to binary compromise for all existing tools (`ama/tool.ts`,
`quality-check/tool.ts`) and is out of scope for this feature.

**Risk (README write-if-missing):** Write-if-missing never overwrites. No escalation possible.

**Verdict:** Risk profile is identical to the existing `ama` and `quality-check` scaffolding.
No new attack surface.

### 3. Path traversal

**Finding:** `paths::tool_dir(root, "dev-driver")` builds the target directory using a
hardcoded tool name string literal `"dev-driver"`. The `paths::tool_dir` function is a simple
`root.join(".sdlc/tools/").join(name)`.

**Risk:** The name is a string literal in the binary, not user input. There is no injection
vector. Path traversal via `"dev-driver"` is not possible.

**Verdict:** No issue.

### 4. Content of the embedded files

**Finding:** The embedded `tool.ts` content includes `spawn('claude', [...], { detached: true })`
and `execSync(...)` calls.

**Risk (existing risk):** The tool spawns Claude CLI and runs shell commands. This is the
intended behavior of the dev-driver tool and was audited in the `dev-driver-tool` feature. The
content embedded here is bit-for-bit identical to the current `.sdlc/tools/dev-driver/tool.ts`.
No new risk introduced.

**Verdict:** No new risk. Pre-existing and accepted.

### 5. Idempotency and atomicity

**Finding:** `io::atomic_write` is used for `tool.ts`, which writes to a temp file and renames
atomically. Partial writes cannot corrupt the file.

**Verdict:** No issue.

## Findings summary

| # | Finding | Severity | Action |
|---|---|---|---|
| F1 | `include_str!` compile-time embedding | None | N/A |
| F2 | File writes follow existing write policies | None | N/A |
| F3 | Path built from string literal, no user input | None | N/A |
| F4 | Embedded tool content calls spawn/execSync | Accepted (pre-existing) | Tracked in dev-driver-tool audit |
| F5 | Atomic write prevents partial file corruption | None (positive) | N/A |

## Verdict

**PASS.** No new security surface introduced. The change follows the same write policies and
implementation patterns as the existing core tool scaffolding. No findings requiring remediation.
