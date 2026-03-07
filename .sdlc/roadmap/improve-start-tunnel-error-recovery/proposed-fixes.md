# Proposed Fixes

## Fix 1: Augment PATH in find_orch_tunnel() (High impact, low risk)

Add common install locations as fallback paths when `which::which` fails:

```rust
pub fn find_orch_tunnel() -> Result<PathBuf, TunnelError> {
    // First try the process PATH
    if let Ok(p) = which::which("orch-tunnel") {
        return Ok(p);
    }
    // Fallback: check common install locations
    let fallback_dirs = [
        "/opt/homebrew/bin",           // macOS ARM brew
        "/usr/local/bin",              // macOS Intel brew / manual install
        dirs::home_dir().map(|h| h.join(".cargo/bin").to_string_lossy().into_owned()),
    ];
    for dir in fallback_dirs.iter().filter_map(|d| match d {
        s => Some(std::path::PathBuf::from(s)),
    }) {
        let candidate = dir.join("orch-tunnel");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(TunnelError::NotFound)
}
```

This solves the stale-PATH problem without requiring a server restart.

## Fix 2: Improve TunnelError::NotFound message (Low effort)

Add a hint about restarting:
> If you just installed orch-tunnel, restart sdlc-server to pick up PATH changes — or the server will check common install locations automatically (after Fix 1).

## Fix 3: Add /api/tunnel/preflight endpoint (Medium effort)

```
GET /api/tunnel/preflight -> { available: bool, path: string | null, error: string | null }
```

Call on NetworkPage mount. If `available: false`, show install instructions inline (not in an error banner) and disable the Start button with a tooltip.

## Fix 4: Frontend error formatting (Low effort)

The current error banner shows raw multi-line text. Parse the error to show:
- A short summary line
- Expandable install instructions
- A 'retry' button (re-checks after user may have installed)

## Priority Order

1. **Fix 1** — solves the actual bug (PATH not refreshed)
2. **Fix 3** — prevents the error from ever appearing (proactive check)
3. **Fix 4** — better UX when error does occur
4. **Fix 2** — belt-and-suspenders for edge cases

**Commit signal:** Fix 1 alone is sufficient to ship. Fixes 2-4 are polish.