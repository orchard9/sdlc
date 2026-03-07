# `sdlc tunnel check` — Final Design

## Command
`sdlc tunnel check [--json]`

New top-level `Tunnel` command group in CLI. Exit 0 = found, exit 1 = not found.

## Three-Tier Discovery

1. **Process PATH** — `which::which("orch-tunnel")`
2. **Login shell PATH** — `$SHELL -lc "echo $PATH"` → `which::which_in()`
3. **Fallback probing** — `/opt/homebrew/bin`, `/usr/local/bin`, `~/.cargo/bin`

After finding: verify with `<binary> --version`.

## Result Types

```rust
pub struct TunnelCheckResult {
    pub installed: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub source: Option<String>,        // "process_path" | "login_shell_path" | "fallback"
    pub process_path_stale: bool,
    pub checked: Vec<CheckedLocation>,
}

pub struct CheckedLocation {
    pub location: String,
    pub found: bool,
}
```

## File Changes

| File | Change |
|------|--------|
| `crates/sdlc-cli/src/main.rs` | Add `Tunnel { subcommand: TunnelSubcommand }` to Commands |
| `crates/sdlc-cli/src/cmd/mod.rs` | Add `pub mod tunnel_cmd;` |
| `crates/sdlc-cli/src/cmd/tunnel_cmd.rs` | New — TunnelSubcommand::Check + output formatting |
| `crates/sdlc-server/src/tunnel.rs` | Upgrade `find_orch_tunnel()` to three-tier, add `check_orch_tunnel() -> TunnelCheckResult` |

## Key Decisions

- Core discovery logic in `sdlc-server::tunnel` (one implementation, two consumers)
- CLI wraps with formatting only
- `find_orch_tunnel()` keeps `Result<PathBuf, TunnelError>` return type for Tunnel::start() compat
- `check_orch_tunnel()` returns richer `TunnelCheckResult` for the CLI command
- No caching — tunnel start is not a hot path
