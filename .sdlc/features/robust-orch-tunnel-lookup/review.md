# Review: Robust orch-tunnel lookup

## Summary

Upgraded `find_orch_tunnel()` in `crates/sdlc-server/src/tunnel.rs` from a single `which::which()` call to a three-tier discovery strategy with structured diagnostics. Added `check_orch_tunnel()` for rich JSON-serializable results.

## Changes Reviewed

| File | Lines Changed | Nature |
|------|--------------|--------|
| `crates/sdlc-server/src/tunnel.rs` | +200 | New functions, structs, tests |

## Findings

### F1: Thread join in timeout path (low risk)
In `read_login_shell_path()`, the `_ = handle.join()` call in the timeout branch could block briefly if the shell process is still running. However, since the child process has already been spawned and the thread is just waiting on `wait_with_output`, this is bounded by the OS reaping the child. The 3-second timeout on the channel receive is the primary guard. Acceptable as-is.

**Action:** Accept — the thread join is best-effort cleanup and the timeout path is already the fallback of a fallback (tier 2 failing gracefully).

### F2: No `is_executable` check on fallback paths
Tier 3 uses `path.is_file()` but does not verify the file is executable. A non-executable file at a fallback path would be returned but fail when spawned.

**Action:** Accept — `Tunnel::start()` will surface a clear `Process` error if the binary can't be executed, and this is an extremely unlikely edge case (a non-executable file named `orch-tunnel` in `/opt/homebrew/bin`).

### F3: `TunnelError::NotFound` variant changed from unit to String
This is a breaking change for any code that pattern-matches on `TunnelError::NotFound`. Verified via grep that no other code in the codebase matches on this variant — the only consumer is `Tunnel::start()` which propagates via `?`.

**Action:** Verified safe — no external consumers.

### F4: serde added to tunnel.rs imports
The `Serialize`/`Deserialize` derives on `TunnelCheckResult` and `CheckedLocation` require `serde` in scope. `serde` was already a workspace dependency for sdlc-server. Clean.

**Action:** No issue.

## Build Verification

- `SDLC_NO_NPM=1 cargo build --all` — clean
- `SDLC_NO_NPM=1 cargo test -p sdlc-server -- tunnel` — 16/16 pass
- `cargo clippy -p sdlc-server -- -D warnings` — zero warnings

## Verdict

Approve. Clean implementation that follows the ponder design, maintains backward compatibility at the `find_orch_tunnel()` call site, and adds comprehensive test coverage.
