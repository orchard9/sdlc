# Design: orch-tunnel migration

## Overview

This is a CLI/backend change with no new UI surfaces. The changes affect how `sdlc ui` starts the tunnel and what happens when the tunnel fails.

---

## 1. CLI Interface Change

### Before

```
sdlc ui [start] [OPTIONS]

Options:
  --port <PORT>       Port to listen on [default: 3141]
  --no-open           Don't open browser automatically
  --tunnel            Open a public tunnel and print a QR code for remote access (requires orch-tunnel)
  --tick-rate <SECS>  Orchestrator tick interval [default: 60]
  --run-actions       Start the orchestrator daemon
```

### After

```
sdlc ui [start] [OPTIONS]

Options:
  --port <PORT>       Port to listen on [default: 3141]
  --no-open           Don't open browser automatically
  --no-tunnel         Disable the public tunnel (default: tunnel starts automatically)
  --tick-rate <SECS>  Orchestrator tick interval [default: 60]
  --run-actions       Start the orchestrator daemon
```

The `--tunnel` flag is replaced with `--no-tunnel`. Default behavior changes from "no tunnel" to "tunnel on."

---

## 2. Graceful Fallback Logic

The current `run_start` function in `ui.rs` returns an error when the tunnel fails. The new behavior:

```
if use_tunnel:
    match Tunnel::start(port, name):
        Ok(tun)  → proceed with tunnel active
        Err(e)   → eprintln!("Warning: orch-tunnel failed to start ({}). Running in local-only mode.", e)
                   → proceed without tunnel (same path as use_tunnel = false)
```

### Affected error cases

| Error | Current behavior | New behavior |
|-------|-----------------|--------------|
| `TunnelError::NotFound` (orch-tunnel not in PATH) | Return error, exit | Warn + local-only mode |
| `TunnelError::Timeout` (edge unreachable) | Return error, exit | Warn + local-only mode |
| `TunnelError::ExitedEarly` (tunnel crash) | Return error, exit | Warn + local-only mode |
| `TunnelError::Process` (spawn failed) | Return error, exit | Warn + local-only mode |

All four error variants now result in a warning message and local-only continuation, not process exit.

---

## 3. `run_start` Control Flow (After)

```
run_start(root, port, no_open, no_tunnel, tick_rate, run_actions):
    auto-update scaffolding
    maybe start orchestrator thread
    load config
    check for existing UI instance
    bind TCP listener
    write registry record

    use_tunnel = !no_tunnel  // default true

    if use_tunnel:
        print warning about public exposure
        try Tunnel::start(port, project_name):
            Ok(tun)  → token = generate_token()
                       print_tunnel_info(name, port, tun.url, token)
                       serve_on(root, listener, false, Some((tun, token)))
            Err(e)   → eprintln!("Warning: orch-tunnel failed to start ({e}). Running in local-only mode.")
                       serve_on(root, listener, !no_open, None)  // fall through to local
    else:
        println!("SDLC UI for '{name}' → {local_url}  (PID {pid})")
        serve_on(root, listener, !no_open, None)
```

---

## 4. Argument Struct Changes

In `UiSubcommand::Start` and in the top-level `sdlc ui` flags (both paths use the same logic):

```rust
// Before
#[arg(long)]
tunnel: bool,

// After
#[arg(long)]
no_tunnel: bool,
```

The `run_start` function signature changes from `use_tunnel: bool` to `no_tunnel: bool`, then internally computes `let use_tunnel = !no_tunnel`.

---

## 5. No Changes Required

- `crates/sdlc-server/src/tunnel.rs` — already implements orch-tunnel correctly
- `crates/sdlc-server/src/auth.rs` — no changes; auth middleware is unaffected
- `crates/sdlc-server/src/routes/tunnel.rs` — no changes needed
- `crates/sdlc-cli/src/cmd/tunnel.rs` — `print_tunnel_info` is unchanged

---

## 6. Terminal Output (After)

### Tunnel success case
```
Warning: tunnel mode exposes your SDLC server publicly. Share the QR code only with trusted parties.

SDLC UI for 'sdlc'
  Local:   http://localhost:3141  (no auth)
  Tunnel:  https://sdlc.tunnel.threesix.ai

  ┌────────────────────────────────┐
  │  [QR CODE]                     │
  └────────────────────────────────┘

  Passcode:  abc12345
  (embedded in QR — scan to access)

Ctrl+C to stop
```

### Tunnel fallback case (orch-tunnel not installed)
```
Warning: tunnel mode exposes your SDLC server publicly. Share the QR code only with trusted parties.
Warning: orch-tunnel failed to start (orch-tunnel not found
...). Running in local-only mode.
SDLC UI for 'sdlc' → http://localhost:3141  (PID 12345)
```

### No-tunnel case
```
SDLC UI for 'sdlc' → http://localhost:3141  (PID 12345)
```

---

## 7. Test Coverage Plan

- Unit test: `run_start` with `no_tunnel=true` — does not attempt Tunnel::start
- Unit test: graceful fallback path — mock Tunnel::start returning error, assert process continues
- Existing tests in `tunnel.rs` remain unchanged
- Integration test not required (tunnel start requires live binary)
