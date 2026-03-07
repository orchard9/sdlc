# Design: Robust orch-tunnel lookup

## Overview

Upgrade the binary discovery in `crates/sdlc-server/src/tunnel.rs` from a single `which::which()` call to a three-tier strategy with structured diagnostics.

## Architecture

All discovery logic stays in `crates/sdlc-server/src/tunnel.rs`. No new crates or modules needed. The CLI and server routes consume the same functions.

```
find_orch_tunnel()          -> Result<PathBuf, TunnelError>   (existing signature, upgraded internals)
check_orch_tunnel()         -> TunnelCheckResult              (new, richer diagnostics)
read_login_shell_path()     -> Option<String>                 (new, internal helper)
```

## Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelCheckResult {
    pub installed: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub source: Option<String>,        // "process_path" | "login_shell_path" | "fallback"
    pub process_path_stale: bool,      // true when found via tier 2/3 but not tier 1
    pub checked: Vec<CheckedLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedLocation {
    pub location: String,
    pub found: bool,
}
```

## Three-Tier Discovery Flow

```
find_orch_tunnel()
  |
  +-- Tier 1: which::which("orch-tunnel")
  |     Found? -> return Ok(path)
  |
  +-- Tier 2: read_login_shell_path() -> which::which_in("orch-tunnel", fresh_path, ".")
  |     Found? -> return Ok(path)
  |
  +-- Tier 3: probe fallback locations
  |     /opt/homebrew/bin/orch-tunnel
  |     /usr/local/bin/orch-tunnel
  |     ~/.cargo/bin/orch-tunnel
  |     Found? -> return Ok(path)
  |
  +-- Err(TunnelError::NotFound) with enriched message
```

### read_login_shell_path()

```
$SHELL (or /bin/sh fallback) -lc "echo $PATH"
  -> capture stdout
  -> trim
  -> return Some(path_string) or None on any failure
```

Timeout: 3 seconds max to prevent hanging on broken shell configs. Use `std::process::Command` (blocking, not async) since this runs once during discovery.

### check_orch_tunnel()

Runs all three tiers regardless of early success to populate the full `TunnelCheckResult`. After finding the binary, runs `<binary> --version` to capture the version string.

## Error Message Improvement

Current `TunnelError::NotFound` is a static string. Change to include the list of locations checked:

```
orch-tunnel not found.

Searched:
  Process PATH:      not found
  Login shell PATH:  not found (shell: /bin/zsh)
  /opt/homebrew/bin: not found
  /usr/local/bin:    not found
  ~/.cargo/bin:      not found

Install:
  macOS    brew install orch-tunnel
  Other    gh release download --repo orchard9/tunnel \
             --pattern 'orch-tunnel-*' -D /usr/local/bin
           chmod +x /usr/local/bin/orch-tunnel

Then re-run: sdlc ui --tunnel
```

To support dynamic messages, change `NotFound` from a unit variant to carry a `String` payload:

```rust
#[error("{0}")]
NotFound(String),
```

## File Changes

| File | Change |
|------|--------|
| `crates/sdlc-server/src/tunnel.rs` | Upgrade `find_orch_tunnel()`, add `check_orch_tunnel()`, `read_login_shell_path()`, structs, tests |
| `crates/sdlc-server/Cargo.toml` | Add `dirs` dependency (for `home_dir()`) if not already present |

## Testing Strategy

- `read_login_shell_path()`: test that it returns `Some(non-empty string)` on a normal system
- `find_orch_tunnel()` tier 3: test fallback probing with a tempdir containing a mock binary
- `check_orch_tunnel()`: test struct population and JSON serialization
- `TunnelError::NotFound`: test that the enriched message includes "Searched:" section
