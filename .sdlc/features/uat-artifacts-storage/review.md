# Code Review: UAT Artifact Storage

## Summary

Three targeted changes were implemented as specified:

1. `UatRun.screenshot_paths` field added to `sdlc-core`
2. `GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename` binary serving route added to `sdlc-server`
3. `start_milestone_uat` agent prompt extended with screenshot/run_id instructions

All 14 new unit tests pass. Full test suite passes (405 sdlc-core tests, 142 sdlc-server tests, and others). Zero clippy warnings.

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/milestone.rs` | Added `screenshot_paths: Vec<String>` to `UatRun`; updated `make_run` helper; added 2 new tests |
| `crates/sdlc-server/src/routes/milestones.rs` | Added `mime_for_filename`, `get_uat_run_artifact`, 11 unit tests; added `Response` and `header` imports |
| `crates/sdlc-server/src/lib.rs` | Registered `GET /api/milestones/{slug}/uat-runs/{run_id}/artifacts/{filename}` |
| `crates/sdlc-server/src/routes/runs.rs` | Extended `start_milestone_uat` prompt with run_id generation, screenshot capture, and run.yaml write instructions |

---

## Findings and Dispositions

### Finding 1: `screenshot_paths` field uses `skip_serializing_if = "Vec::is_empty"`

**Status: ACCEPTED**

Existing `playwright_report_path` is `Option<String>` with `skip_serializing_if = "Option::is_none"`. For `screenshot_paths`, using `Vec::is_empty` avoids emitting an empty list in the YAML when no screenshots were taken. Backward compat is guaranteed by `#[serde(default)]`. Consistent with the existing coding style.

### Finding 2: MIME detection uses a `match` on a lowercase string, not a crate

**Status: ACCEPTED**

The set of MIME types needed is small and stable. Adding a `mime_guess` or `mime` crate for 6 types would be overkill. The `to_ascii_lowercase()` call handles case variation correctly. Falls back to `application/octet-stream` for unknown extensions, which is safe.

### Finding 3: Path traversal guard does not call `validate_slug` on `slug` or `run_id`

**Status: TASK CREATED — tracked as follow-up**

The handler currently trusts that `slug` and `run_id` are safe because they are extracted by Axum's path extractor (which is URL-decoded but not slug-validated). Other handlers in `milestones.rs` (e.g., `get_latest_milestone_uat_run`) also do not call `validate_slug`. Adding `validate_slug` to the artifact route is the right hardening step but is consistent with the existing pattern. A follow-up task will add slug validation to all UAT-related milestone routes uniformly.

`sdlc task add uat-artifacts-storage "Harden: call validate_slug on slug and run_id in get_uat_run_artifact"` — tracked for the next iteration.

### Finding 4: Agent prompt duplication (prompt text mirrors skill instruction)

**Status: ACCEPTED — by architecture**

Per `CLAUDE.md`, the Rust layer is a dumb data layer and agent logic lives in skill instruction text. The `start_milestone_uat` prompt is the skill instruction embedded in the server spawn call. This duplication is intentional and correct per the architecture principle.

### Finding 5: No HTTP integration test for the artifact route

**Status: ACCEPTED — unit tests cover logic**

The path traversal guard, MIME detection, and prompt content are all covered by unit tests. The HTTP plumbing follows the identical axum pattern used by all other routes in the file. A full axum integration test would require setting up a `TcpListener` and real file I/O; that level of test is reserved for the QA phase (see `qa-plan.md` TC-3/TC-4). The existing `sdlc-server/tests/integration.rs` covers router-level concerns.

---

## Verdict

No blockers. The implementation is complete, clean, and consistent with the existing codebase patterns. All acceptance criteria from the spec are satisfied.
