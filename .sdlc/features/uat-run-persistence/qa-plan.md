# QA Plan: uat-run-persistence

## Strategy

Pure library code with no UI or HTTP surface — QA is entirely via `cargo test` and `cargo clippy`.

## Test Coverage

| Test | What it validates |
|---|---|
| `uat_run_round_trip` | `save_uat_run` writes a valid `run.yaml`; `list_uat_runs` deserializes it and returns a matching `UatRun` |
| `uat_run_list_sorted_newest_first` | Two runs with different `started_at` — list returns newest first |
| `uat_run_latest_none_when_empty` | `latest_uat_run` on a milestone with no `uat-runs/` dir returns `Ok(None)` |

## Quality Gates

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass, zero failures
- `cargo clippy --all -- -D warnings` — zero warnings
- No `unwrap()` in non-test code paths (enforced by code review in the review artifact)

## Edge Cases

- `list_uat_runs` when `uat-runs/` directory does not exist → returns empty `Vec`
- `list_uat_runs` when a subdirectory exists but has no `run.yaml` → skipped silently
- `save_uat_run` creates all parent directories automatically via `atomic_write`

## Regression Risk

Low. All changes are additive — new types and functions only, no modifications to existing `Milestone` methods or existing file paths.
