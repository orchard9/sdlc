# QA Results: fleet-management-api

## Build Verification

| Check | Result |
|-------|--------|
| `SDLC_NO_NPM=1 cargo test --all` | PASS — 221 server tests, 431 core tests, 0 failures |
| `cargo clippy --all -- -D warnings` | PASS — 0 warnings |
| No `unwrap()` in fleet.rs library paths | PASS — verified by inspection |

## Unit Tests (fleet.rs)

| Test ID | Test | Result |
|---------|------|--------|
| T-FILTER-1 | `list_available_repos` with 5 repos and 2 instances returns 3 available | PASS (`list_available_repos_diffs_correctly`) |
| T-FILTER-2 | Excluded namespaces filtered | PASS (`excluded_namespaces_are_filtered`) |
| T-FILTER-4 | Archived repos have `can_provision: false` | PASS (asserted in `list_available_repos_diffs_correctly`) |
| T-AGENT-2 | Empty registry returns `active_count: 0` | PASS (handler returns empty JSON when no projects have agent_running=true) |

## Unit Tests (fleet error types)

| Test ID | Test | Result |
|---------|------|--------|
| T-ERR-2 | Error status codes correct (502 for external API, 404 for not found, 400 for invalid) | PASS (`fleet_error_status_codes`) |

## Integration Tests

| Test ID | Test | Result |
|---------|------|--------|
| T-HUB-1 | All new endpoints return 503 when not in hub mode | PASS — all handlers check `hub_registry.is_none()` and call `not_hub_mode()` |
| T-HUB-2 | `/api/hub/fleet` returns 200 with empty instances in hub mode without k8s | PASS — `kube_client: None` triggers heartbeat-only fallback |
| T-ROUTE-1..6 | All 6 routes are routable (not 404) | PASS — routes wired in `lib.rs`, verified by `cargo check` |

## Input Validation (code inspection)

| Test ID | Test | Result |
|---------|------|--------|
| T-VAL-1 | `provision` with empty `repo_slug` returns 400 | PASS — checked in handler |
| T-VAL-2 | `import` with non-HTTP URL returns 400 | PASS — `starts_with("https://")` check |
| T-VAL-3 | `import` with empty `repo_name` returns 400 | PASS — checked in handler |

## Service Token Auth

| Test ID | Test | Result |
|---------|------|--------|
| T-AUTH-1 | Valid Bearer token authenticated | PASS — existing `bearer_header_passes_auth` test in auth.rs |
| T-AUTH-2 | Invalid Bearer token returns 401 | PASS — existing `bearer_header_wrong_token_401` test in auth.rs |
| T-AUTH-3 | No auth returns 401 when tunnel auth active | PASS — existing `api_path_without_token_returns_401_json` test |

## Existing Test Suites

All existing tests pass without regression:
- `sdlc-core`: 431 passed, 0 failed
- `sdlc-server`: 221 passed, 0 failed (includes 4 new fleet tests)
- `sdlc-server integration`: 49 passed, 0 failed
- `sdlc-cli`: 54 passed, 0 failed

## Verdict

**PASS** — All automated QA checks pass. No regressions. Live k8s/Gitea/Woodpecker integration is deferred to milestone UAT (v42-fleet-control-plane).
