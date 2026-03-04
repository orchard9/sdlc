# Design: human-uat-backend

## Architecture

This is a pure backend feature. No UI changes (that is `human-uat-frontend`). The changes span two crates:

```
sdlc-core/src/milestone.rs
  └── UatRunMode enum (new)
  └── UatRun.mode field (new, serde default = Agent)

sdlc-server/src/routes/runs.rs
  └── submit_milestone_uat_human() handler (new)

sdlc-server/src/routes/features.rs
  └── submit_human_qa() handler (new)

sdlc-server/src/lib.rs
  └── two new route registrations

sdlc-core/src/lib.rs
  └── re-export UatRunMode
```

## Data Model Change

### `UatRunMode` (new enum in `milestone.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UatRunMode {
    #[default]
    Agent,
    Human,
}
```

### `UatRun.mode` (new field)

```rust
pub struct UatRun {
    // ... existing fields unchanged ...
    #[serde(default)]
    pub mode: UatRunMode,
}
```

`serde(default)` uses `UatRunMode::Agent` for any existing `run.yaml` file that lacks the field — fully backward-compatible.

## New Route: `POST /api/milestone/{slug}/uat/human`

**Location:** `crates/sdlc-server/src/routes/runs.rs`

**Request body struct:**
```rust
#[derive(serde::Deserialize)]
pub struct HumanUatBody {
    pub verdict: UatVerdict,           // "pass" | "pass_with_tasks" | "failed"
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub notes: String,
}
```

**Validation:**
- `verdict` is required (serde handles this)
- `notes` must be non-empty when `verdict` is `Failed` or `PassWithTasks`

**Handler logic (pseudocode):**
```
1. validate slug
2. spawn_blocking:
   a. load Milestone (→ 404 if not found)
   b. validate notes (→ 422 if empty for non-pass verdicts)
   c. generate run_id: "{YYYYMMDD}-{HHMMSS}-{3 random lowercase letters}"
   d. now = Utc::now()
   e. summary_path = format!(".sdlc/milestones/{slug}/uat-runs/{run_id}/summary.md")
   f. write summary.md via atomic_write
   g. build UatRun { ..., mode: UatRunMode::Human, ... }
   h. save_uat_run(root, &run)
   i. if verdict == Pass: milestone.release(); milestone.save(root)
3. emit SseMessage::MilestoneUatCompleted { slug }
4. return { run_id, slug, status: "submitted" }
```

**Summary.md template:**
```
# UAT Results — {slug}

Run ID: {run_id}
Mode: Human (manual)
Verdict: {verdict_display}
Tests: {tests_passed}/{tests_total} passed

## Notes
{notes}

Submitted: {timestamp}
```

## New Route: `POST /api/features/{slug}/human-qa`

**Location:** `crates/sdlc-server/src/routes/features.rs`

**Request body struct:**
```rust
#[derive(serde::Deserialize)]
pub struct HumanQaBody {
    pub verdict: String,   // "pass" | "pass_with_tasks" | "failed"
    pub notes: String,
}
```

**Validation:**
- `verdict` must be one of the three accepted strings (→ 422 otherwise)
- `notes` must be non-empty when verdict is not `"pass"`

**Handler logic (pseudocode):**
```
1. validate slug
2. spawn_blocking:
   a. load Feature (→ 404 if not found)
   b. validate verdict string, validate notes
   c. format qa-results.md content
   d. write to .sdlc/features/{slug}/qa-results.md via atomic_write
   e. Feature::load again (re-read after write to ensure consistency)
   f. feature.draft_artifact(ArtifactType::QaResults)
   g. feature.save(root)
3. emit SseMessage::Update (triggers frontend refresh)
4. return { slug, artifact: "qa_results", status: "draft" }
```

**qa-results.md template:**
```markdown
## Verdict
{verdict_display}

## Notes
{notes}

Runner: human (manual)
Completed: {timestamp}
```

## Route Registrations in `lib.rs`

```rust
.route(
    "/api/milestone/{slug}/uat/human",
    post(routes::runs::submit_milestone_uat_human),
)
.route(
    "/api/features/{slug}/human-qa",
    post(routes::features::submit_human_qa),
)
```

The `/api/milestone/{slug}/uat/human` route sits alongside the existing `/api/milestone/{slug}/uat` (agent) route. It does not conflict with `/api/milestone/{slug}/uat/stop` or `/fail` because "human" is a distinct literal segment.

## run_id Generation

Use `chrono` (already a dependency) for the timestamp portion and `rand` (already in `sdlc-server`) for the 3-letter suffix:

```rust
use rand::Rng;

fn generate_run_id() -> String {
    let ts = Utc::now().format("%Y%m%d-%H%M%S");
    let suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .filter(|c| c.is_ascii_lowercase())
        .take(3)
        .map(char::from)
        .collect();
    format!("{ts}-{suffix}")
}
```

## Integration Test Plan

Tests live in `crates/sdlc-server/tests/integration.rs`, following existing patterns.

| Test | Setup | Assertion |
|---|---|---|
| `human_uat_submit_pass` | create milestone, POST with verdict=pass | 200, run.yaml has mode=human, milestone released_at set |
| `human_uat_submit_pass_with_tasks_empty_notes` | create milestone, POST with verdict=pass_with_tasks, notes="" | 422 |
| `human_uat_submit_failed_empty_notes` | create milestone, POST verdict=failed, notes="" | 422 |
| `human_qa_submit_drafts_artifact` | create feature in qa phase, POST to human-qa | 200, qa_results artifact is draft |
| `uat_run_mode_backward_compat` | deserialize YAML without mode field | UatRunMode::Agent |

## Dependencies

- `rand` — already in `sdlc-server/Cargo.toml`
- `chrono` — already in both crates
- No new dependencies required

## Error Handling

All handlers return `Result<Json<serde_json::Value>, AppError>`. Validation errors use `AppError` wrapping `anyhow::anyhow!("...")` with appropriate HTTP status codes. The `AppError` type must support 422 responses — check existing usage and add if needed.

```rust
// 422 pattern (to add if AppError doesn't already support it)
return Err(AppError::unprocessable("notes are required when verdict is not pass"));
```

If `AppError` only supports `anyhow` with 500, we map validation failures to a descriptive 422 response via a `ValidationError` newtype returning `StatusCode::UNPROCESSABLE_ENTITY`.
