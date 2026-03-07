# Tasks: Robust orch-tunnel lookup

## T1: Add read_login_shell_path() helper
Add `read_login_shell_path() -> Option<String>` to `crates/sdlc-server/src/tunnel.rs`. Spawns `$SHELL -lc "echo $PATH"` (fallback `/bin/sh`) with a 3-second timeout. Returns the trimmed PATH string or None on any failure.

## T2: Upgrade find_orch_tunnel() to three-tier discovery
Replace the single `which::which()` call with the three-tier strategy: process PATH, login shell PATH, fallback probing of `/opt/homebrew/bin`, `/usr/local/bin`, `~/.cargo/bin`. Change `TunnelError::NotFound` to carry an enriched diagnostic message listing all locations checked.

## T3: Add TunnelCheckResult structs and check_orch_tunnel()
Add `TunnelCheckResult`, `CheckedLocation` structs with Serialize/Deserialize. Implement `check_orch_tunnel()` that runs all three tiers and captures version via `--version`. Make structs `pub` for future CLI/endpoint consumption.

## T4: Add unit tests
Test `read_login_shell_path()` returns a non-empty string, test fallback probing with a temp mock binary, test `TunnelCheckResult` JSON serialization, test enriched NotFound message content. Ensure `SDLC_NO_NPM=1 cargo test --all` passes.
