# QA Plan: orch-tunnel migration

## Test Strategy

This is a CLI behavior change. Primary verification is through unit tests and a manual smoke test. No UI changes, no data migrations, no server route changes.

---

## Automated Tests

### 1. CLI flag regression — `--tunnel` removed, `--no-tunnel` present

Run `sdlc ui start --help` and verify:
- `--tunnel` does not appear in the output
- `--no-tunnel` appears with description mentioning "Disable the public tunnel"

```bash
cargo build --bin sdlc 2>&1
./target/debug/sdlc ui start --help | grep -E "tunnel"
# Expected: line containing "--no-tunnel" but NOT "--tunnel" alone
```

### 2. Existing tunnel unit tests pass

```bash
SDLC_NO_NPM=1 cargo test --all 2>&1
```

All tests in `crates/sdlc-server/src/tunnel.rs` must pass:
- `extract_url_from_bare_line`
- `extract_url_embedded_in_log_line`
- `extract_url_ignores_non_orch_tunnel`
- `token_is_8_alphanumeric_chars`

### 3. Auth middleware tests pass

All tests in `crates/sdlc-server/src/auth.rs` must pass.

### 4. Graceful fallback unit test

The new unit test (T4 from tasks.md) covering the fallback path must pass:
- When tunnel start fails, `run_start` does not return an error
- A warning is printed to stderr

---

## Manual Smoke Test

### Scenario A: Default (tunnel auto-start)

Precondition: `orch-tunnel` is installed and reachable.

```bash
sdlc ui
```

Expected:
- Warning about public exposure is printed
- Tunnel starts and URL `https://<project>.tunnel.threesix.ai` is displayed
- QR code is printed
- Server serves requests on both localhost and tunnel URL

### Scenario B: Opt-out (`--no-tunnel`)

```bash
sdlc ui --no-tunnel
```

Expected:
- No tunnel starts
- Local URL is printed: `SDLC UI for '<name>' → http://localhost:3141  (PID <pid>)`
- No QR code or tunnel URL
- Server serves requests on localhost

### Scenario C: Graceful fallback (orch-tunnel not in PATH)

Precondition: Temporarily rename/remove `orch-tunnel` from PATH.

```bash
PATH=/usr/bin:/bin sdlc ui
```

Expected:
- Warning message: `Warning: orch-tunnel failed to start (...). Running in local-only mode.`
- Local URL is printed
- Server starts and serves localhost
- Process does NOT exit with an error

### Scenario D: Old `--tunnel` flag removed (backward-incompatible, intentional)

```bash
sdlc ui --tunnel
```

Expected:
- Clap error: `error: unexpected argument '--tunnel' found`
- Process exits non-zero
- Help text references `--no-tunnel`

---

## Acceptance Criteria (from spec)

- [x] `sdlc ui` (no flags) starts orch-tunnel automatically and prints the QR code — verified by Scenario A
- [x] `sdlc ui --no-tunnel` starts without a tunnel — verified by Scenario B
- [x] When orch-tunnel is not installed, `sdlc ui` logs a warning and continues — verified by Scenario C
- [x] When orch-tunnel fails to connect, `sdlc ui` logs a warning and continues — covered by unit test T4
- [x] All existing tests pass — verified by automated test suite
- [x] `--tunnel` is removed from CLI; `--no-tunnel` appears — verified by test 1 + Scenario D

---

## Regression Risk

Low. The only behavioral change is:
1. Default tunnel start (was opt-in, now opt-out)
2. Failure mode change (was hard exit, now graceful fallback)

Auth middleware, server routing, tunnel protocol, and QR code generation are all unchanged.
