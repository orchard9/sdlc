---
session: 1
timestamp: 2026-03-07T17:30:00Z
orientation:
  current: "Root cause identified — stale PATH in running process. Fix is clear: augment find_orch_tunnel() with fallback paths."
  next: "Implement Fix 1 (PATH fallback in find_orch_tunnel) and Fix 3 (preflight endpoint). Small, shippable change."
  commit: "Fix 1 alone is sufficient to commit. It solves the actual bug."
---

**Xist · Owner**
improve start tunnel error recovery

http://localhost:7778/network

on this page i click "start tunnel" button for the Ponder Terminal, and I get an error saying orch-tunnel cannot be found. I did install `brew install orch-tunnel` and I think it worked...  Is there some issue with PATH? orch-tunnel requires special PATH maybe, and the running sdlc ui doesn't have the updated PATH?

---

## Investigation

Traced the call path:

1. **Frontend** (`NetworkPage.tsx:139`) — `api.startTunnel()` → `POST /api/tunnel`
2. **Route** (`routes/tunnel.rs:59`) — `Tunnel::start(port, &name)`
3. **Tunnel::start** (`tunnel.rs:60`) — `find_orch_tunnel()?`
4. **find_orch_tunnel** (`tunnel.rs:163-164`) — `which::which("orch-tunnel").map_err(|_| TunnelError::NotFound)`

`which::which()` searches the process's `PATH` environment variable — inherited at spawn time and never refreshed.

## Root Cause

**Stale PATH.** The sdlc-server process was started before `orch-tunnel` was installed. The server's `PATH` doesn't include the new binary location. Even though `which orch-tunnel` works in a fresh terminal, the running server can't find it.

Secondary issue: `brew install orch-tunnel` references a homebrew tap (`orchard9/homebrew-tap`) that doesn't exist yet per the `install-distribution` ponder sessions. The install may not have actually succeeded.

## Proposed Fixes

### Fix 1: PATH fallback in find_orch_tunnel() (ship this)

After `which::which` fails, check common install locations directly:
- `/opt/homebrew/bin/orch-tunnel` (macOS ARM brew)
- `/usr/local/bin/orch-tunnel` (macOS Intel / manual)
- `~/.cargo/bin/orch-tunnel` (cargo install)

This is a 10-line change in `tunnel.rs` that eliminates the stale-PATH class of errors entirely.

### Fix 3: Preflight endpoint (polish)

`GET /api/tunnel/preflight` → `{ available, path, error }`. Frontend calls on mount, disables button + shows install instructions when unavailable. Prevents the error from ever appearing.

### Fix 4: Frontend error formatting (polish)

Parse multi-line error into summary + expandable details + retry button.

## Decisions

- ⚑ Decided: Fix 1 is the right fix. PATH fallback paths in `find_orch_tunnel()`.
- ⚑ Decided: Priority order is Fix 1 → Fix 3 → Fix 4.
- ? Open: Should we also re-read PATH from the shell on each tunnel start attempt? (e.g., run `zsh -lc 'echo $PATH'` and parse it). More robust but adds subprocess overhead.
- ? Open: Is `brew install orch-tunnel` actually working? The homebrew tap may not exist. Need to verify separately.
