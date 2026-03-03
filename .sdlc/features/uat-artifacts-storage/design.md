# Design: UAT Artifact Storage

## Overview

Three targeted changes that together give UAT runs a persistent visual record:

1. **Model extension** — add `screenshot_paths` to `UatRun`
2. **Binary serving route** — new Axum handler streams files from the run directory
3. **Agent prompt fix** — `start_milestone_uat` gains explicit screenshot-saving instructions

---

## 1. UatRun Model Extension

**File:** `crates/sdlc-core/src/milestone.rs`

Add one field to `UatRun`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UatRun {
    pub id: String,
    pub milestone_slug: String,
    pub started_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub verdict: UatVerdict,
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playwright_report_path: Option<String>,
    #[serde(default)]
    pub tasks_created: Vec<String>,
    pub summary_path: String,
    /// Relative paths (from project root) to screenshots captured during this run.
    /// Stored as `.sdlc/milestones/<slug>/uat-runs/<id>/<filename>`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshot_paths: Vec<String>,
}
```

`#[serde(default)]` on `screenshot_paths` ensures existing YAML files without this field deserialize without error.

---

## 2. Binary Serving Route

**File:** `crates/sdlc-server/src/routes/milestones.rs` (new handler)
**Registration:** `crates/sdlc-server/src/lib.rs` or equivalent router

### Route

```
GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename
```

### Handler logic

```rust
pub async fn get_uat_run_artifact(
    State(app): State<AppState>,
    Path((slug, run_id, filename)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    // 1. Path traversal guard
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::bad_request("invalid filename"));
    }

    // 2. Build the path
    let path = paths::uat_run_dir(&app.root, &slug, &run_id).join(&filename);

    // 3. Read the file
    let bytes = tokio::fs::read(&path).await
        .map_err(|_| AppError::not_found("artifact not found"))?;

    // 4. Detect MIME type
    let content_type = mime_for_filename(&filename);

    // 5. Return the response
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .body(Body::from(bytes))
        .unwrap())
}

fn mime_for_filename(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png"  => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webm" => "video/webm",
        "mp4"  => "video/mp4",
        "gif"  => "image/gif",
        _      => "application/octet-stream",
    }
}
```

### Router registration

```rust
.route(
    "/api/milestones/:slug/uat-runs/:run_id/artifacts/:filename",
    get(routes::milestones::get_uat_run_artifact),
)
```

---

## 3. Agent Prompt Fix

**File:** `crates/sdlc-server/src/routes/runs.rs`, function `start_milestone_uat`

The current prompt instructs the agent to run checklist steps and write `uat_results.md`, but says nothing about saving screenshots. The fix adds explicit screenshot instructions.

### Current prompt (abbreviated)

```
Call `sdlc milestone info {slug} --json` to load the milestone and acceptance test.
Execute every checklist step. Write signed checklist results to
`.sdlc/milestones/{slug}/uat_results.md`.
Then call `sdlc milestone complete {slug}` if all steps pass.
```

### Updated prompt additions

```
Before starting: generate a run ID using the format `YYYYMMDD-HHMMSS-<random-3-chars>` (UTC).
Create the run directory: `.sdlc/milestones/{slug}/uat-runs/<run_id>/`.

For every checklist step that involves a UI interaction:
  - After completing the step, call `mcp__playwright__browser_take_screenshot` with
    filename `<step_number>-<slug_of_step>.png`.
  - The Playwright MCP will return the screenshot path. Copy the file to
    `.sdlc/milestones/{slug}/uat-runs/<run_id>/<step_number>-<slug_of_step>.png`.
  - Collect the relative path `.sdlc/milestones/{slug}/uat-runs/<run_id>/<filename>`
    in a `screenshot_paths` list.

After all steps complete, write `run.yaml` to
`.sdlc/milestones/{slug}/uat-runs/<run_id>/run.yaml` using `sdlc uat-run save` (or
directly write the YAML via the Write tool following the UatRun schema):
  id, milestone_slug, started_at, completed_at, verdict, tests_total, tests_passed,
  tests_failed, tasks_created, summary_path, screenshot_paths.
```

---

## Data Flow

```
UAT agent
  ├── sdlc milestone info <slug> --json         # load checklist
  ├── generate run_id                            # YYYYMMDD-HHMMSS-abc
  ├── for each step:
  │     ├── browser_navigate / browser_click …
  │     └── browser_take_screenshot → copy to uat-runs/<run_id>/<step>.png
  ├── write summary.md → uat-runs/<run_id>/summary.md
  ├── write run.yaml → uat-runs/<run_id>/run.yaml  (screenshot_paths populated)
  └── write uat_results.md → milestones/<slug>/uat_results.md

Frontend (uat-artifacts-ui)
  └── GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename
        → binary PNG bytes
```

---

## File Changes Summary

| File | Change |
|---|---|
| `crates/sdlc-core/src/milestone.rs` | Add `screenshot_paths: Vec<String>` to `UatRun` |
| `crates/sdlc-server/src/routes/milestones.rs` | Add `get_uat_run_artifact` handler |
| `crates/sdlc-server/src/lib.rs` | Register new route |
| `crates/sdlc-server/src/routes/runs.rs` | Extend `start_milestone_uat` prompt |

---

## Error Handling

- `filename` with `/`, `\`, or `..` → `400 Bad Request`
- File not found on disk → `404 Not Found`
- `slug` or `run_id` with invalid slug chars → `400 Bad Request` (existing `validate_slug`)
- All errors propagate as `AppError` per existing server convention
