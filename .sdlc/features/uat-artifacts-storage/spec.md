# Spec: UAT Artifact Storage

## Feature

**Slug:** `uat-artifacts-storage`
**Title:** UAT artifact storage — UatRun model extension, binary serving route, agent prompt fix
**Milestone:** `v19-uat-artifacts` — UAT Artifacts: screenshots and video evidence for every run

## Problem

UAT runs produce visual evidence — screenshots taken by the Playwright MCP browser — but there is currently no place to store them, no way to retrieve them, and the UAT agent prompt does not instruct the agent to save artifacts into the run directory. Screenshots captured during a UAT run are ephemeral: they exist only in the Playwright MCP's local session and are never persisted to `.sdlc/`.

This means:
- UAT runs have no visual audit trail
- `UatRun` records carry a `playwright_report_path` field but it's never populated
- The agent prompt for `start_milestone_uat` says nothing about saving screenshots
- The frontend (`uat-artifacts-ui`, companion feature) has no route to fetch binary artifacts

## Goals

1. **Extend `UatRun` model** — add `screenshot_paths: Vec<String>` (relative to project root) to track every screenshot captured during a run.
2. **Add binary-serving route** — `GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename` streams a file from `.sdlc/milestones/<slug>/uat-runs/<run_id>/` back to the caller. Supports PNG, JPEG, WebM (and any other MIME type via extension detection). Enforces path-traversal protection.
3. **Fix agent prompt** — update `start_milestone_uat` in `runs.rs` to instruct the agent to: (a) save screenshots into `.sdlc/milestones/<slug>/uat-runs/<run_id>/` using a timestamp-based filename, and (b) record each path in `UatRun.screenshot_paths` via `save_uat_run`.

## Non-Goals

- Video recording (out of scope for this feature; `v19` milestone can add later)
- Frontend display of screenshots (covered by `uat-artifacts-ui`)
- Playwright report HTML (separate artifact type; paths already tracked in `playwright_report_path`)
- Auth on the artifact route (the server already has tunnel auth middleware)

## Acceptance Criteria

1. `UatRun` struct has `screenshot_paths: Vec<String>` field, `serde(default)`, and serializes/deserializes correctly via existing YAML round-trip tests.
2. `GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename` returns the file bytes with correct `Content-Type` for `.png`, `.jpg`, `.webm`, and returns 404 for missing files. Returns 400 if path traversal is attempted (e.g., `../../../etc/passwd`).
3. The route is registered in `crates/sdlc-server/src/lib.rs` (or equivalent router setup).
4. The agent prompt in `start_milestone_uat` instructs the agent to:
   - Call `mcp__playwright__browser_take_screenshot` with a filename of the form `<timestamp>-<step>.png`
   - Save the file to `.sdlc/milestones/<slug>/uat-runs/<run_id>/<filename>`
   - After all steps complete, call `save_uat_run` with the updated `screenshot_paths` list
5. Existing `UatRun` YAML files (without `screenshot_paths`) deserialize correctly (backward compat via `serde(default)`).
6. `SDLC_NO_NPM=1 cargo test --all` passes with the new field and route.
7. `cargo clippy --all -- -D warnings` produces zero warnings.

## Implementation Notes

- The run ID format used by the agent is currently free-form (e.g., `2026-03-02-per-feature-qa`). The prompt fix should instruct the agent to generate a run ID before starting and use it consistently throughout.
- The `save_uat_run` function in `sdlc-core/src/milestone.rs` already handles directory creation — no changes needed there.
- MIME detection: use a simple `match` on extension. No external crate needed.
- Path traversal: reject any filename containing `/`, `\`, or `..`.
- The binary route should use `tokio::fs::read` and return `axum::response::Response` with the bytes body.
