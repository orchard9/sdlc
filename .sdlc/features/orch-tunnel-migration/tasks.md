# Tasks: orch-tunnel migration

## T1 — Flip `--tunnel` to `--no-tunnel` in `UiSubcommand::Start`

In `crates/sdlc-cli/src/cmd/ui.rs`:
- Remove `#[arg(long)] tunnel: bool` from `UiSubcommand::Start`
- Add `#[arg(long)] no_tunnel: bool` to `UiSubcommand::Start`
- Remove `#[arg(long)] tunnel: bool` from the top-level `sdlc ui` flags in `main.rs` (or wherever the top-level args are parsed — trace back from `run()`)
- Add `#[arg(long)] no_tunnel: bool` to the top-level flags
- Update `run()` dispatch and `run_start()` signature accordingly: `use_tunnel: bool` → `no_tunnel: bool`, then `let use_tunnel = !no_tunnel;`

## T2 — Implement graceful tunnel fallback in `run_start`

In `crates/sdlc-cli/src/cmd/ui.rs`, change the tunnel error handling in `run_start`:

Current:
```rust
Err(TunnelError::NotFound) => {
    let _ = record_clone.remove();
    return Err(anyhow!("{}", TunnelError::NotFound));
}
Err(e) => {
    let _ = record_clone.remove();
    return Err(anyhow!("{e}"));
}
```

New:
```rust
Err(e) => {
    eprintln!("Warning: orch-tunnel failed to start ({e}). Running in local-only mode.");
    // Fall through — serve local-only, no early return
    println!("SDLC UI for '{name}' → {local_url}  (PID {pid})");
    tokio::select! {
        res = sdlc_server::serve_on(root_buf, listener, !no_open, None) => res,
        _ = tokio::signal::ctrl_c() => Ok(()),
    }
}
```

Both `TunnelError::NotFound` and all other errors now fall through to local-only mode instead of removing the registry record and exiting.

## T3 — Update the help text / documentation

- Update the arg help string for `--no-tunnel`: `"Disable the public tunnel (tunnel starts automatically by default, requires orch-tunnel)"`
- Update `docs/orch-tunnel-reference.md` to reflect:
  - Default behavior is now tunnel-on
  - Use `--no-tunnel` to run without a tunnel
  - Graceful fallback behavior when orch-tunnel is unavailable

## T4 — Add unit test for graceful fallback in ui.rs

The existing unit tests in `tunnel.rs` cover URL extraction and token generation. Add a test (or integration test) that exercises the fallback code path:
- Mock/simulate `Tunnel::start` returning an error
- Assert that `run_start` does not return an error
- Assert that a warning was printed to stderr

Note: This may need to be a doc-test or integration test depending on how `run_start` is structured. If the async runtime setup is complex, a simpler approach is a dedicated `#[test]` that tests the fallback logic in isolation (extracting the match arm into a helper function).
