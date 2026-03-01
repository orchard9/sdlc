# Design: uat-run-persistence

## Summary

Add `UatRun` / `UatVerdict` to `crates/sdlc-core/src/milestone.rs` and three path helpers to `paths.rs`. No new modules required — the data and functions are small enough to live alongside the existing `Milestone` code.

## Module Placement

All new code goes into existing files:

| File | What changes |
|---|---|
| `crates/sdlc-core/src/paths.rs` | 3 new path helpers: `milestone_uat_runs_dir`, `uat_run_dir`, `uat_run_manifest` |
| `crates/sdlc-core/src/milestone.rs` | `UatVerdict` enum, `UatRun` struct, 3 free functions |

No changes to `lib.rs` are required unless other crates need to import `UatRun` / `UatVerdict` directly (the server and CLI will import via `sdlc_core::milestone::UatRun`).

## Directory Layout

```
.sdlc/
  milestones/
    <milestone-slug>/
      manifest.yaml
      acceptance_test.md      ← existing
      uat_results.md          ← existing (single-file legacy, kept as-is)
      uat-runs/
        <run-id>/
          run.yaml            ← NEW: serialized UatRun
          summary.md          ← written by the UAT skill, not by this feature
```

`<run-id>` format: `YYYYMMDD-HHMMSS-<6-char-hex>` — constructed by the caller, stored verbatim in `UatRun.id`.

## Data Types

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
    pub id: String,
    pub milestone_slug: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub verdict: UatVerdict,
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub playwright_report_path: Option<String>,
    pub tasks_created: Vec<String>,
    pub summary_path: String,
}
```

## Path Helpers

```rust
// paths.rs additions
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

## Functions

### `save_uat_run`

```rust
pub fn save_uat_run(root: &Path, run: &UatRun) -> Result<()> {
    let path = paths::uat_run_manifest(root, &run.milestone_slug, &run.id);
    let data = serde_yaml::to_string(run)?;
    crate::io::atomic_write(&path, data.as_bytes())
}
```

`atomic_write` calls `create_dir_all` on the parent, so no explicit directory creation is needed.

### `list_uat_runs`

```rust
pub fn list_uat_runs(root: &Path, milestone_slug: &str) -> Result<Vec<UatRun>> {
    let dir = paths::milestone_uat_runs_dir(root, milestone_slug);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut runs = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let run_id = entry.file_name().to_string_lossy().into_owned();
        let manifest = paths::uat_run_manifest(root, milestone_slug, &run_id);
        if manifest.exists() {
            let data = std::fs::read_to_string(&manifest)?;
            let run: UatRun = serde_yaml::from_str(&data)?;
            runs.push(run);
        }
    }
    runs.sort_by(|a, b| b.started_at.cmp(&a.started_at)); // newest first
    Ok(runs)
}
```

### `latest_uat_run`

```rust
pub fn latest_uat_run(root: &Path, milestone_slug: &str) -> Result<Option<UatRun>> {
    Ok(list_uat_runs(root, milestone_slug)?.into_iter().next())
}
```

## Error Handling

All errors propagate via `?` to the `Result<_, SdlcError>` return type. `SdlcError` already covers `Io`, `Yaml`, and other variants from upstream crates.

## Tests

Three tests in the `#[cfg(test)]` block at the bottom of `milestone.rs`:

| Test | Assertion |
|---|---|
| `uat_run_round_trip` | save then list returns the same run |
| `uat_run_list_sorted_newest_first` | two runs with different timestamps → newest first |
| `uat_run_latest_none_when_empty` | `latest_uat_run` returns `Ok(None)` on empty dir |

Tests use `tempfile::TempDir` exactly as the existing milestone tests do.

## Non-Goals

- No CLI subcommand for UAT runs in this feature — the persistence layer only.
- No server route — the `sdlc-milestone-uat` skill writes runs directly via Rust library calls or CLI.
- No migration of existing `uat_results.md` files.
