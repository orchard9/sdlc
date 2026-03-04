# Code Review: orch-tunnel migration

## Summary

This review covers the implementation of the orch-tunnel migration feature. Four tasks were completed:

- T1: Flip `--tunnel` to `--no-tunnel` in `UiSubcommand::Start` and update `run_start` signature
- T2: Implement graceful tunnel fallback in `run_start`
- T3: Update help text and `docs/orch-tunnel-reference.md`
- T4: Add unit tests for `no_tunnel` flag semantics and fallback warning format

A pre-existing compile error in `crates/sdlc-server/src/routes/tunnel.rs` (from the `auth-named-tokens` partial work) was also fixed as part of this pass — `get_tunnel` referenced the old `snap.config.token` field that was renamed to `tokens: Vec<(String, String)>` in the working tree.

---

## Files Changed

| File | Change |
|------|--------|
| `crates/sdlc-cli/src/cmd/ui.rs` | `--tunnel` → `--no-tunnel`, graceful fallback, unit tests |
| `crates/sdlc-cli/src/main.rs` | `--tunnel` → `--no-tunnel` in `Commands::Ui` variant |
| `crates/sdlc-server/src/routes/tunnel.rs` | Fix `token` → new `TunnelConfig` API (pre-existing breakage) |
| `docs/orch-tunnel-reference.md` | Updated to reflect default-on behavior and `--no-tunnel` |

---

## Correctness Review

### T1 — Flag rename

The `--tunnel: bool` → `--no-tunnel: bool` change in both `UiSubcommand::Start` (ui.rs) and the top-level `Commands::Ui` (main.rs) is complete and consistent. The `run()` dispatch and `run_start()` signature are updated everywhere the parameter is threaded.

The semantic inversion is correctly computed at the entry point of `run_start`:
```rust
let use_tunnel = !no_tunnel;
```

This keeps all downstream logic using `use_tunnel` (true = tunnel active), which is clearer than threading `no_tunnel` through the entire function.

### T2 — Graceful fallback

The new fallback path removes `record_clone.remove()` before returning an error — the process now continues, so there is no reason to remove the registry record. This is correct.

The fallback prints the local URL and invokes `serve_on` with `None` for the tunnel, identical to the explicit `--no-tunnel` path. No new codepath, just reusing the existing local-only serving branch.

The warning message includes the full error (`{e}`) so operators see exactly what failed (binary not found, timeout, etc.).

### T3 — Docs

`docs/orch-tunnel-reference.md` now has a "Default Behavior" section at the top explaining: tunnel starts by default, `--no-tunnel` to disable, graceful fallback if unavailable. The `stderr` → `stdout` reference was also corrected (orch-tunnel outputs the URL to stdout, not stderr).

### T4 — Tests

Two focused unit tests were added to `cmd::ui::tests`:

1. `no_tunnel_flag_inverts_use_tunnel` — verifies the boolean inversion contract
2. `fallback_warning_format_is_informative` — verifies the warning message contains expected substrings

These test the two semantically important behaviors without requiring a full async server stack.

### Tunnel.rs fix

The `get_tunnel` handler at line 32 referenced `snap.config.token` (old single-token field). The new `TunnelConfig` uses `tokens: Vec<(String, String)>`. The fix removes the stale token field access from GET and returns `None` consistently — tokens are never returned on GET in the new auth model (only on POST when first generated). This is also more secure.

---

## Quality Checks

- `cargo build -p sdlc-cli` — passes
- `cargo build -p sdlc-server` — passes
- `SDLC_NO_NPM=1 cargo test --all` — 893 tests pass, 0 failures
- `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings` — 0 warnings

---

## Spec Compliance

| Criterion | Status |
|-----------|--------|
| `sdlc ui` (no flags) starts tunnel automatically | PASS — `use_tunnel = !no_tunnel` defaults to true |
| `sdlc ui --no-tunnel` runs without tunnel | PASS — `no_tunnel=true` → `use_tunnel=false` |
| orch-tunnel not installed → warn and continue | PASS — all `Err(e)` arms now fall through to local-only |
| orch-tunnel fails → warn and continue | PASS — same `Err(e)` catch-all handles timeout, exit early, etc. |
| All existing tests pass | PASS — 893 tests, 0 failures |
| `--tunnel` removed from help; `--no-tunnel` present | PASS — arg renamed in both struct and dispatch |
| Tunnel URL stable across restarts | PASS — tunnel.rs already uses named tunnels (unchanged) |

---

## Findings and Resolutions

**Finding 1: Pre-existing compile error in routes/tunnel.rs**
- Severity: Blocker (prevented full test suite from passing)
- Root cause: `auth-named-tokens` partial work changed `TunnelConfig.token: Option<String>` to `tokens: Vec<(String, String)>` but did not update `get_tunnel` which referenced `.token`
- Action: Fixed now — `get_tunnel` returns `token: None` consistently (tokens not exposed on GET)
- Task tracked: N/A (fix-forward in this review pass)

No other findings.
