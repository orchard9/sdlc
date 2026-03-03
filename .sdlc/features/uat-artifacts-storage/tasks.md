# Tasks: UAT Artifact Storage

## T1 — Extend UatRun with screenshot_paths

**File:** `crates/sdlc-core/src/milestone.rs`

Add `screenshot_paths: Vec<String>` field to `UatRun` struct with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.

Acceptance: existing `uat_run_round_trip` test still passes; a new test demonstrates that a YAML blob without `screenshot_paths` deserializes with an empty vec, and one with it round-trips correctly.

---

## T2 — Add binary artifact serving route

**Files:**
- `crates/sdlc-server/src/routes/milestones.rs` — add `get_uat_run_artifact` handler
- `crates/sdlc-server/src/lib.rs` — register `GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename`

Handler must:
- Reject filenames with `/`, `\`, or `..` (400)
- Return 404 for missing files
- Return correct `Content-Type` for `.png`, `.jpg`/`.jpeg`, `.webm`, `.mp4`, `.gif`; `application/octet-stream` for all others
- Stream file bytes using `tokio::fs::read`

Acceptance: the route is reachable and returns a PNG file correctly; path-traversal attempts return 400.

---

## T3 — Fix start_milestone_uat agent prompt

**File:** `crates/sdlc-server/src/routes/runs.rs`

Extend the `prompt` string in `start_milestone_uat` to instruct the agent to:
1. Generate a run ID (`YYYYMMDD-HHMMSS-<3-chars>`) before starting
2. After each checklist step's UI interaction, take a screenshot with `mcp__playwright__browser_take_screenshot` using a `<step_num>-<step_slug>.png` filename
3. Save each screenshot into `.sdlc/milestones/<slug>/uat-runs/<run_id>/`
4. After all steps, write `run.yaml` to the run directory with all fields populated, including `screenshot_paths`

Acceptance: the prompt string contains the words "run_id", "screenshot", and "screenshot_paths".

---

## T4 — Verify build and tests pass

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All tests pass, zero clippy warnings.
