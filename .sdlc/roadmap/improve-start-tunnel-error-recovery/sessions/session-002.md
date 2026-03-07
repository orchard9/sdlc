---
session: 2
timestamp: 2026-03-07T21:00:00Z
orientation:
  current: "Command design decided: `sdlc tunnel check [--json]` with login-shell PATH refresh + fallback probing. Implementation spec is concrete and ready to build."
  next: "Implement `sdlc tunnel check` — add TunnelSubcommand to CLI, implement check_orch_tunnel() with login-shell PATH + fallbacks, wire --json output."
  commit: "Commit signal already met in session 1. This session refines the design for immediate implementation."
---

**Xist · Owner**
add a sdlc command that will discover whether orch-tunnel is installed or not. this tool must work even if orch-tunnel PATH is not correctly configured.

---

## Design Discussion

### Command Placement

Three candidates from session 1:
1. `sdlc ui check` — under the UI subcommand
2. `sdlc doctor` — standalone top-level diagnostic
3. `sdlc tunnel check` — new tunnel subcommand group

**Dan Reeves:** "Does this need a new top-level command? You already have `sdlc ui` with `start`, `list`, `kill`, `open`. A `check` subcommand there is natural — tunnel is a UI concern. But... the owner said 'sdlc command that will discover whether orch-tunnel is installed.' That's not a UI operation. It's a diagnostic. Putting it under `ui` means a user who just installed orch-tunnel and wants to verify has to know that tunnel lives under `ui`. That's coupling."

**Kai Yamazaki:** "The first thing a user does after `brew install orch-tunnel` is verify it worked. The command they'll reach for is something with 'tunnel' in it. `sdlc tunnel check` is discoverable — `sdlc --help` shows `tunnel`, user explores, finds `check`. `sdlc ui check` buries it. And `sdlc doctor` is too broad for a single binary check — it implies a full system health scan. Don't promise what you won't deliver."

> Decided: **`sdlc tunnel check [--json]`** — new top-level `Tunnel` command group with a `Check` subcommand. Clean namespace, discoverable, extensible (can add `tunnel status`, `tunnel install` later without restructuring).

### Discovery Strategy

The core requirement: find orch-tunnel even when the running process's PATH is stale.

Three-tier search:

1. **Process PATH** — `which::which("orch-tunnel")`. Fast path, works when PATH is correct.
2. **Login shell PATH** — spawn `$SHELL -lc "echo $PATH"`, then `which::which_in()` against the result. Catches the stale-PATH case (server started before brew install).
3. **Known fallback locations** — probe these paths directly:
   - `/opt/homebrew/bin/orch-tunnel` (macOS ARM brew)
   - `/usr/local/bin/orch-tunnel` (macOS Intel / manual install)
   - `$HOME/.cargo/bin/orch-tunnel` (cargo install)
   - `/nix/var/nix/profiles/default/bin/orch-tunnel` (nix)

**Kai Yamazaki:** "After finding the binary, you MUST verify it executes. A dangling symlink or wrong-architecture binary will pass `is_file()` but fail at runtime. Run `<path> --version` and capture output. If it fails, report `found_but_broken: true` with the error. This is the difference between 'installed' and 'working'."

> Decided: After locating the binary, run `<binary> --version` to verify it's executable and capture the version string.

### Output Format

**Human-readable (default):**
```
orch-tunnel: installed
  Path:    /opt/homebrew/bin/orch-tunnel
  Version: 0.2.1
  Source:  login_shell_path (process PATH was stale)
```

Or when not found:
```
orch-tunnel: not found

  Checked:
    - process PATH: not found
    - login shell PATH: not found
    - /opt/homebrew/bin/orch-tunnel: not found
    - /usr/local/bin/orch-tunnel: not found
    - ~/.cargo/bin/orch-tunnel: not found

  Install:
    macOS:  brew install orchard9/tap/orch-tunnel
    Other:  gh release download --repo orchard9/tunnel --pattern 'orch-tunnel-*' -D /usr/local/bin
```

**JSON (`--json`):**
```json
{
  "installed": true,
  "path": "/opt/homebrew/bin/orch-tunnel",
  "version": "0.2.1",
  "source": "login_shell_path",
  "process_path_stale": true
}
```

Or:
```json
{
  "installed": false,
  "checked": [
    {"location": "process_path", "found": false},
    {"location": "login_shell_path", "found": false},
    {"location": "/opt/homebrew/bin/orch-tunnel", "found": false},
    {"location": "/usr/local/bin/orch-tunnel", "found": false},
    {"location": "/Users/xist/.cargo/bin/orch-tunnel", "found": false}
  ],
  "install_hint": "brew install orchard9/tap/orch-tunnel"
}
```

### Implementation Plan

**Files to modify:**

1. **`crates/sdlc-cli/src/main.rs`** — Add `Tunnel { subcommand: TunnelSubcommand }` variant to `Commands` enum, wire dispatch.

2. **`crates/sdlc-cli/src/cmd/mod.rs`** — Add `pub mod tunnel_cmd;` (can't shadow the existing `tunnel.rs` which is QR rendering).

3. **`crates/sdlc-cli/src/cmd/tunnel_cmd.rs`** (new) — `TunnelSubcommand::Check` implementation:
   - `check_orch_tunnel() -> TunnelCheckResult` — the three-tier search
   - `read_login_shell_path() -> Option<String>` — spawn `$SHELL -lc "echo $PATH"`
   - `verify_binary(path: &Path) -> Option<String>` — run `--version`, return version string
   - Human and JSON output formatting

4. **`crates/sdlc-server/src/tunnel.rs`** — Upgrade `find_orch_tunnel()` to use the same three-tier logic. This function is called by `Tunnel::start()` on every tunnel launch, so it fixes the runtime bug too.

**Dan Reeves:** "Don't duplicate the search logic. Put the core `find_orch_tunnel_enhanced()` in `sdlc-server/src/tunnel.rs` (where the runtime caller already lives), have it return a richer result type. The CLI command calls the same function and formats the output. One search implementation, two consumers."

> Decided: Core discovery logic lives in `sdlc-server::tunnel`. CLI imports and wraps with formatting.

### Key Design Choice: Where the Logic Lives

Per the project's "Rust = Data" principle, the search function is pure data — it probes paths and returns what it found. No heuristics, no decisions about what to do next. The CLI formats and prints. This is correct placement.

? Open: Should `find_orch_tunnel()` cache the result? If the server calls it on every tunnel start, the login-shell spawn adds ~50ms latency. Probably fine — tunnel start is not a hot path. Skip caching.

? Open: On Linux, should we also check `/usr/bin/orch-tunnel` and `/snap/bin/orch-tunnel`? Not urgent — orch-tunnel is macOS-focused today.

### Interaction with Existing `find_orch_tunnel()`

The current `find_orch_tunnel()` at `crates/sdlc-server/src/tunnel.rs:163` is:
```rust
pub fn find_orch_tunnel() -> Result<PathBuf, TunnelError> {
    which::which("orch-tunnel").map_err(|_| TunnelError::NotFound)
}
```

This gets replaced with the three-tier version. The return type stays `Result<PathBuf, TunnelError>` for backward compatibility with `Tunnel::start()`. The richer `TunnelCheckResult` struct is a separate function used by the CLI check command.

### Exit Code

`sdlc tunnel check` exits 0 when found, exits 1 when not found. This makes it scriptable:
```bash
if sdlc tunnel check --json > /dev/null 2>&1; then
  sdlc ui
else
  echo "Install orch-tunnel first"
fi
```

> Decided: Exit code 0 = found, 1 = not found. Standard diagnostic convention.
