# Design: `sdlc tunnel check`

## Command

`sdlc tunnel check [--json]`

New subcommand under `sdlc ui` or standalone `Tunnel` top-level variant — adds a `Check` subcommand to `UiSubcommand`.

Actually: add it as a new top-level command: **`sdlc doctor`** — broader than just tunnel. But for now, the minimal version is a `Check` variant on `UiSubcommand`:

```
sdlc ui check [--json]
```

## What it does

1. **Re-read PATH from login shell** — runs `zsh -lc "echo \$PATH"` (or `bash -lc` as fallback) to get the real, current PATH. This is the key insight: the running process PATH may be stale.

2. **Search for orch-tunnel** using the refreshed PATH via `which::which_in("orch-tunnel", fresh_path, ".")`.

3. **Fallback to known locations** if login-shell PATH also fails:
   - `/opt/homebrew/bin/orch-tunnel` (macOS ARM)
   - `/usr/local/bin/orch-tunnel` (macOS Intel / manual)
   - `~/.cargo/bin/orch-tunnel` (cargo install)

4. **Verify the binary is executable** — not just that the path exists, but that it runs (`orch-tunnel --version` or similar).

5. **Report result** as JSON or human-readable:
   ```json
   {
     "installed": true,
     "path": "/opt/homebrew/bin/orch-tunnel",
     "version": "0.1.0",
     "source": "login_shell_path",
     "process_path_stale": true
   }
   ```
   or:
   ```json
   {
     "installed": false,
     "checked_paths": [...],
     "install_instructions": "..."
   }
   ```

## Implementation location

- `find_orch_tunnel()` in `crates/sdlc-server/src/tunnel.rs` — enhance to use login-shell PATH + fallbacks
- New function `check_orch_tunnel() -> CheckResult` — returns structured info (not just PathBuf)
- CLI wiring: add `Check` to `UiSubcommand` in `cmd/ui.rs`

## Key: does NOT require specific PATH

The `zsh -lc` approach reads the user's full `.zprofile` / `.zshrc` PATH, catching any brew/cargo/nix additions. The fallback paths cover the case where even that fails.

## Prerequisite finding

`brew install orch-tunnel` does NOT work today:
- No `orchard9/homebrew-tap` tap exists
- `brew info orch-tunnel` returns "No available formula"
- The install instructions in TunnelError::NotFound are currently misleading

This needs to be addressed separately (create the homebrew tap, or change install instructions to point to GitHub releases).
