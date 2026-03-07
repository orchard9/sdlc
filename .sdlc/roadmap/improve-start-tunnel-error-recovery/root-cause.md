# Root Cause Analysis

## The Problem

When a user clicks 'Start tunnel' on the Network page and `orch-tunnel` isn't found, the error recovery is poor:

1. **PATH inheritance** — `find_orch_tunnel()` (`tunnel.rs:163`) uses `which::which("orch-tunnel")` which only searches the running process's `PATH`. If `sdlc-server` was started before `orch-tunnel` was installed, the server's `PATH` doesn't include the new binary location. Restarting the server is required but the error doesn't say this.

2. **Homebrew tap doesn't exist yet** — `brew install orch-tunnel` requires `orchard9/homebrew-tap` which hasn't been created (confirmed in `install-distribution` ponder session-003). The install instructions reference a non-existent formula.

3. **Error message is raw text** — The `TunnelError::NotFound` error is a multi-line plaintext string with install instructions. The frontend displays it in a small error banner, making it hard to read. No structured guidance.

4. **No pre-flight visibility** — The UI has no way to know if `orch-tunnel` is available before the user clicks. The button looks ready but will always fail.

## The PATH Problem in Detail

`which::which()` searches `std::env::var("PATH")` — the process environment at spawn time. Common scenario:
- User runs `cargo run` or `sdlc ui` from terminal
- User installs orch-tunnel in another terminal (new PATH entry)
- User clicks 'Start tunnel' in browser → fails
- User is confused because `which orch-tunnel` works in their new terminal

This is the exact scenario Xist hit.