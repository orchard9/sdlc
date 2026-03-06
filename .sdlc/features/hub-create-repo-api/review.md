# Code Review: hub-create-repo-api

## Changes

- `crates/sdlc-server/src/fleet.rs` — added `FleetError::RepoAlreadyExists`, `create_gitea_repo()`, `get_gitea_username()`, 4 unit tests
- `crates/sdlc-server/src/routes/hub.rs` — added `POST /api/hub/create-repo` handler with `CreateRepoRequest`
- `crates/sdlc-server/src/lib.rs` — registered new route
- `crates/sdlc-server/Cargo.toml` — added `mockito = "1"` dev-dependency

## Findings

### PASS — Correctness

`create_gitea_repo` correctly maps HTTP 409 to `RepoAlreadyExists` and all other non-success status codes to `GiteaUnavailable`. The pattern matches `import_repo` exactly.

`create_repo` handler correctly:
- Guards hub mode and Gitea config before any work
- Validates name (non-empty, lowercase alphanum + hyphens, max 100 chars)
- Builds push URL with embedded credentials using correct scheme detection
- Calls `trigger_provision` as fire-and-forget (warns on failure, doesn't fail the request)
- Returns all fields the frontend needs: `repo_slug`, `push_url`, `gitea_url`, `provision_triggered`

### PASS — Error handling

All error paths return appropriate status codes via the existing `FleetError::into_response()` pattern. No new error handling patterns introduced.

### PASS — Tests

4 new async tests cover: successful create, 409 conflict, successful username fetch, and 401 username error. All use `mockito` for HTTP mocking. All pass.

### PASS — No unwrap in library code

`get_gitea_username` uses `unwrap_or_default()` on the scheme detection — acceptable for a string literal comparison. All other paths use `?` or explicit `match`.

### PASS — Build

`SDLC_NO_NPM=1 cargo build -p sdlc-server` — clean.
`cargo clippy -p sdlc-server -- -D warnings` — no warnings.
8 fleet tests pass.

### MINOR — Push URL exposes admin token in plaintext

The push URL embeds the admin Gitea token in plaintext HTTP basic auth. This is intentional per the spec (internal single-operator tool) and documented in the spec rationale. Future multi-user scenario should use per-repo deploy keys or per-user tokens.

No action required — accepted per spec decision.

## Verdict

APPROVED. All tasks implemented correctly, tests pass, no regressions.
