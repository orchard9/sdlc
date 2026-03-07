# QA Plan: Robust orch-tunnel lookup

## Automated Tests

### Unit tests (cargo test)
1. `read_login_shell_path()` returns `Some` with a non-empty string on the test machine
2. `find_orch_tunnel()` finds a mock binary placed in a fallback location (tempdir)
3. `check_orch_tunnel()` populates all fields of `TunnelCheckResult` including `checked` list
4. `TunnelCheckResult` serializes to valid JSON with expected field names
5. `TunnelError::NotFound` message contains "Searched:" and lists checked locations
6. Login shell fallback to `/bin/sh` when `$SHELL` is unset

### Integration verification
7. `SDLC_NO_NPM=1 cargo test --all` passes with zero failures
8. `cargo clippy --all -- -D warnings` passes with zero warnings

## Manual Verification (for reviewer)

9. Start `sdlc-server`, confirm `find_orch_tunnel()` still finds orch-tunnel via tier 1 when it is on PATH
10. Unset PATH entry for orch-tunnel, confirm tier 2 (login shell) or tier 3 (fallback) finds it
11. Remove orch-tunnel entirely, confirm enriched error message lists all locations checked

## Pass Criteria

- All automated tests pass
- No clippy warnings
- `find_orch_tunnel()` return type is unchanged (`Result<PathBuf, TunnelError>`)
- Existing `Tunnel::start()` call site requires no changes
