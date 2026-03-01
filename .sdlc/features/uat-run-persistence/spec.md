# Feature Specification: uat-run-persistence

## Overview

Add a `UatRun` struct and `UatVerdict` enum to `crates/sdlc-core` to persist structured UAT run records under `.sdlc/milestones/<slug>/uat-runs/<id>/`. This provides a durable, scannable history of UAT executions for any milestone.

## Problem Statement

Currently, milestone UAT state is stored as a single `uat_results.md` markdown file, which is overwritten on each run. There is no structured history of UAT runs, no machine-readable verdict, and no way to query the latest or list all past runs. The `sdlc-milestone-uat` command has no persistence layer to write its results to.

## Solution

Add `UatRun` (struct) and `UatVerdict` (enum) to `crates/sdlc-core/src/milestone.rs`. Add three free functions or methods:
- `save_uat_run(root, run)` — persist one run
- `list_uat_runs(root, milestone_slug)` — scan and return all runs, newest first
- `latest_uat_run(root, milestone_slug)` — convenience wrapper returning the newest or None

Storage path: `.sdlc/milestones/<slug>/uat-runs/<id>/run.yaml`

## Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UatVerdict {
    Pass,
    PassWithTasks,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UatRun {
    pub id: String,                              // "20260228-143022-abc123"
    pub milestone_slug: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub verdict: UatVerdict,
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub playwright_report_path: Option<String>,  // relative to .sdlc/
    pub tasks_created: Vec<String>,
    pub summary_path: String,                    // relative: milestones/<slug>/uat-runs/<id>/summary.md
}
```

## Functions

### `save_uat_run(root: &Path, run: &UatRun) -> Result<()>`
- Path: `<root>/.sdlc/milestones/<run.milestone_slug>/uat-runs/<run.id>/run.yaml`
- Uses `crate::io::atomic_write` (creates parent dirs automatically)
- Serializes with `serde_yaml`

### `list_uat_runs(root: &Path, milestone_slug: &str) -> Result<Vec<UatRun>>`
- Scans `<root>/.sdlc/milestones/<milestone_slug>/uat-runs/`
- Returns empty Vec if directory does not exist
- Deserializes each `run.yaml`; skips non-directory entries
- Returns sorted descending by `started_at` (newest first)

### `latest_uat_run(root: &Path, milestone_slug: &str) -> Result<Option<UatRun>>`
- Calls `list_uat_runs` and returns `first()`

## Path Constants

Add to `crates/sdlc-core/src/paths.rs`:
```rust
pub fn milestone_uat_runs_dir(root: &Path, slug: &str) -> PathBuf {
    milestone_dir(root, slug).join("uat-runs")
}

pub fn uat_run_dir(root: &Path, milestone_slug: &str, run_id: &str) -> PathBuf {
    milestone_uat_runs_dir(root, milestone_slug).join(run_id)
}

pub fn uat_run_manifest(root: &Path, milestone_slug: &str, run_id: &str) -> PathBuf {
    uat_run_dir(root, milestone_slug, run_id).join("run.yaml")
}
```

## Integration Tests

In `milestone.rs` `#[cfg(test)]` block:

1. **round_trip**: `save_uat_run` then `list_uat_runs` returns exactly the saved run with matching fields.
2. **list_sorted_newest_first**: save two runs with different `started_at`, assert list order is newest first.
3. **latest_returns_none_when_empty**: `latest_uat_run` on a milestone with no runs returns `Ok(None)`.

## Constraints

- No `unwrap()` in library code — use `?` and `SdlcError`
- All file writes go through `crate::io::atomic_write`
- Use `chrono::DateTime<Utc>` for timestamps
- Use `serde_yaml` for serialization
- Re-export `UatRun` and `UatVerdict` from `lib.rs` if needed by server or CLI crates

## Acceptance Criteria

1. `save_uat_run` creates the `run.yaml` file at the correct path
2. `list_uat_runs` returns all saved runs, sorted newest-first
3. `latest_uat_run` returns `None` when no runs exist
4. All three integration tests pass under `SDLC_NO_NPM=1 cargo test --all`
5. No `unwrap()` calls in non-test code
6. `cargo clippy --all -- -D warnings` produces zero warnings
