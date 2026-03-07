# Spec: Robust orch-tunnel lookup

## Problem

`find_orch_tunnel()` in `crates/sdlc-server/src/tunnel.rs` uses `which::which("orch-tunnel")` which only searches the process's inherited `PATH`. When `sdlc-server` is started before `orch-tunnel` is installed (or installed in a different shell session), the binary is invisible despite being available on the system. The user sees a `NotFound` error with no actionable diagnostic.

This is the exact scenario users hit: install orch-tunnel in one terminal, click "Start tunnel" in the UI, get a confusing error even though `which orch-tunnel` works in their current shell.

## Solution

Replace the single `which::which()` call in `find_orch_tunnel()` with a three-tier discovery strategy:

1. **Process PATH** — `which::which("orch-tunnel")` (fast path, current behavior)
2. **Login shell PATH** — spawn `$SHELL -lc "echo $PATH"` to capture the user's real PATH, then `which::which_in()` against it
3. **Fallback probing** — check well-known install locations: `/opt/homebrew/bin/orch-tunnel`, `/usr/local/bin/orch-tunnel`, `~/.cargo/bin/orch-tunnel`

Additionally, add a `check_orch_tunnel()` function that returns a structured `TunnelCheckResult` with:
- Whether the binary was found
- The resolved path
- The version string (from `--version`)
- Which tier found it (`process_path`, `login_shell_path`, `fallback`)
- Whether the process PATH is stale (found via tier 2/3 but not tier 1)
- All locations checked

## Scope

### In scope
- Upgrade `find_orch_tunnel()` to three-tier discovery in `crates/sdlc-server/src/tunnel.rs`
- Add `read_login_shell_path()` helper
- Add `check_orch_tunnel() -> TunnelCheckResult` for richer diagnostics
- Add `TunnelCheckResult` and `CheckedLocation` structs
- Improve `TunnelError::NotFound` message with diagnostic info (which locations were checked)
- Unit tests for the new discovery logic

### Out of scope
- CLI `sdlc tunnel check` command (separate feature or follow-up)
- `GET /api/tunnel/preflight` endpoint (belongs to `tunnel-preflight-check` feature)
- Frontend UI changes (belongs to `tunnel-preflight-check` feature)
- Homebrew tap creation

## Acceptance criteria

1. When orch-tunnel is on the process PATH, behavior is unchanged (tier 1 fast path)
2. When orch-tunnel is NOT on process PATH but IS on the login shell PATH, it is found via tier 2
3. When orch-tunnel is in a well-known location but not on any PATH, it is found via tier 3
4. When orch-tunnel is not found anywhere, the error message lists all locations checked
5. `check_orch_tunnel()` returns a structured result suitable for JSON serialization
6. Login shell PATH resolution gracefully handles missing `$SHELL`, shell spawn failures, and non-zero exits
7. All new code has unit tests; `SDLC_NO_NPM=1 cargo test --all` passes
