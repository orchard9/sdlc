# Implementation Plan

## Phase 1: Enhance `find_orch_tunnel()` (Fix 1 from session 1)

**File:** `crates/sdlc-server/src/tunnel.rs`

Replace the current single-line `find_orch_tunnel()` with:

```rust
pub fn find_orch_tunnel() -> Result<PathBuf, TunnelError> {
    // 1. Try process PATH first (fast path)
    if let Ok(p) = which::which("orch-tunnel") {
        return Ok(p);
    }

    // 2. Re-read PATH from login shell
    if let Some(fresh_path) = read_login_shell_path() {
        if let Ok(p) = which::which_in("orch-tunnel", Some(fresh_path), ".") {
            return Ok(p);
        }
    }

    // 3. Check known fallback locations
    let home = dirs::home_dir().unwrap_or_default();
    let fallbacks = [
        PathBuf::from("/opt/homebrew/bin/orch-tunnel"),
        PathBuf::from("/usr/local/bin/orch-tunnel"),
        home.join(".cargo/bin/orch-tunnel"),
    ];
    for path in &fallbacks {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    Err(TunnelError::NotFound)
}

fn read_login_shell_path() -> Option<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    std::process::Command::new(&shell)
        .args(["-lc", "echo $PATH"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}
```

**Dependency:** Add `dirs` to sdlc-server Cargo.toml (or use `std::env::var("HOME")`).

## Phase 2: Add `sdlc ui check` command

**File:** `crates/sdlc-cli/src/cmd/ui.rs`

Add a `Check` variant to `UiSubcommand`:

```rust
#[derive(Subcommand)]
pub enum UiSubcommand {
    /// Check if orch-tunnel is installed and discoverable
    Check,
}
```

Implementation calls `check_orch_tunnel()` which returns structured info and prints JSON or human table.

## Phase 3: Preflight endpoint

**File:** `crates/sdlc-server/src/routes/tunnel.rs`

Add `GET /api/tunnel/preflight`:
```rust
pub async fn tunnel_preflight() -> Json<PreflightResult> {
    // calls check_orch_tunnel() — same logic as CLI
}
```

Frontend calls this on mount of the Network page, disables "Start Tunnel" button if unavailable, shows install instructions.

## Order of execution

1. Phase 1 — fixes the actual bug (stale PATH)
2. Phase 2 — gives agents and users a way to diagnose
3. Phase 3 — prevents the error from appearing in the UI at all
