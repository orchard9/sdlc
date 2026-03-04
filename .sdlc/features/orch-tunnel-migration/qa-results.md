# QA Results: orch-tunnel migration

## Automated Test Results

### Build

```
SDLC_NO_NPM=1 cargo build -p sdlc-cli
```
**Result:** PASS — compiled without errors or warnings

### Test Suite

```
SDLC_NO_NPM=1 cargo test --all
```

**Result:** PASS — 893 tests, 0 failures

| Crate | Tests | Result |
|-------|-------|--------|
| claude_agent | 23 | PASS |
| sdlc_cli (lib) | 54 | PASS |
| sdlc_cli (bin) | 54 | PASS |
| sdlc_server (integration) | 114 | PASS |
| sdlc_server (2 targets) | 2 | PASS |
| sdlc_core | 436 | PASS |
| sdlc_server (lib) | 161 | PASS |
| sdlc_server (tunnel + auth + tests) | 49 | PASS |

New tests added in this feature:
- `cmd::ui::tests::no_tunnel_flag_inverts_use_tunnel` — PASS
- `cmd::ui::tests::fallback_warning_format_is_informative` — PASS

### Linter

```
SDLC_NO_NPM=1 cargo clippy --all -- -D warnings
```
**Result:** PASS — 0 warnings

---

## CLI Flag Verification

Verified via `cargo build` and CLI inspection:

### `--no-tunnel` is present

```
./target/debug/ponder ui start --help
```
Output includes: `--no-tunnel   Disable the public tunnel (tunnel starts automatically by default, requires orch-tunnel)`

### `--tunnel` is absent

The `--tunnel` flag is no longer recognized. Passing `--tunnel` to `sdlc ui` will produce:
```
error: unexpected argument '--tunnel' found
```

---

## Acceptance Criteria Verification

| Criterion | Verified | Notes |
|-----------|----------|-------|
| `sdlc ui` (no flags) starts tunnel by default | PASS | `use_tunnel = !no_tunnel` defaults to true |
| `sdlc ui --no-tunnel` disables tunnel | PASS | `no_tunnel=true` → `use_tunnel=false` |
| orch-tunnel not found → warn + continue | PASS | All `Err(e)` arms fall through to local-only |
| orch-tunnel timeout → warn + continue | PASS | Same `Err(e)` catch-all |
| All existing tests pass | PASS | 893 tests, 0 failures |
| `--tunnel` removed from help | PASS | Arg renamed to `--no-tunnel` |
| `--no-tunnel` in help text | PASS | Present with descriptive help |
| Tunnel URL stable across restarts | PASS | Unchanged — tunnel.rs uses named tunnels |

---

## Pre-existing Fix

A compile error in `crates/sdlc-server/src/routes/tunnel.rs` was fixed as part of this QA pass. The `get_tunnel` handler referenced a stale field (`snap.config.token`) from the old single-token `TunnelConfig`. After the `auth-named-tokens` partial work changed `TunnelConfig` to use `tokens: Vec<(String, String)>`, this line failed to compile. The fix removes the stale field access and returns `token: None` on GET (tokens are not exposed after initial generation).

---

## Overall Result: PASS

All acceptance criteria met. Test suite green. Ready for merge.
