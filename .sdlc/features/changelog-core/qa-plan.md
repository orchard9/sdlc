# QA Plan: changelog-core

## Scope

Validate the changelog event log core implementation: the `event_log.rs` module, all 5 emission call sites, the SSE `ChangelogUpdated` notification, and the mtime watcher.

## Test approach

Primary: automated Rust unit tests in `event_log.rs` and integration tests using `tempfile::TempDir`. Secondary: manual smoke test by running `sdlc merge` on a test feature and verifying `.sdlc/changelog.yaml` is created.

## Test cases

### TC-1: Append/query round-trip

**Setup**: Empty temp dir with `.sdlc/` directory.
**Action**: Call `append_event` 3 times with distinct events.
**Assert**: `query_events` returns 3 entries; fields match exactly (kind, slug, timestamp, details).
**Pass criteria**: Round-trip is lossless for all 7 EventKind variants.

### TC-2: Newest-first ordering

**Setup**: Append event at t0, then at t1 (t1 > t0).
**Action**: `query_events(root, None, None)`.
**Assert**: First entry has timestamp t1, second has t0.
**Pass criteria**: Newest-first order is enforced.

### TC-3: `since` filter

**Setup**: Append event at t0, sleep 1ms, record t_boundary, append event at t1.
**Action**: `query_events(root, Some(t_boundary), None)`.
**Assert**: Returns only the t1 event.
**Pass criteria**: Events at or before `since` are excluded.

### TC-4: `limit` cap

**Setup**: Append 5 events.
**Action**: `query_events(root, None, Some(2))`.
**Assert**: Returns exactly 2 events (the newest 2).
**Pass criteria**: Limit is respected.

### TC-5: Empty file returns Ok(vec![])

**Setup**: Create empty `.sdlc/changelog.yaml` (zero bytes).
**Action**: `query_events(root, None, None)`.
**Assert**: Returns `Ok(vec![])` — no panic, no error.
**Pass criteria**: Graceful empty-file handling.

### TC-6: Missing file returns Ok(vec![])

**Setup**: Temp dir with no changelog.yaml.
**Action**: `query_events(root, None, None)`.
**Assert**: Returns `Ok(vec![])`.
**Pass criteria**: Graceful missing-file handling.

### TC-7: All 7 EventKind variants serialize correctly

**Setup**: Create one ChangeEvent for each of the 7 EventKind variants.
**Action**: `append_event` all 7, then `query_events`.
**Assert**: All 7 are present; `kind` field matches the expected snake_case string in the YAML.
**Pass criteria**: `feature_merged`, `run_failed`, `milestone_wave_completed`, `feature_phase_advanced`, `review_approved`, `audit_approved`, `qa_approved`.

### TC-8: Append is non-destructive

**Setup**: Append event A. Read file content. Append event B.
**Assert**: File still contains event A's data and event B's data. Event A was not overwritten.
**Pass criteria**: Append never truncates or rewrites existing content.

### TC-9: Changelog write failure does not abort parent operation

**Setup**: Make `.sdlc/` directory read-only so `append_event` will fail.
**Action**: Run the append; verify caller doesn't panic or propagate the error (at emission call sites that suppress errors).
**Assert**: Parent operation (e.g., merge) completes successfully; warning is printed.
**Pass criteria**: Non-fatal emission behavior confirmed.

### TC-10: `extract_slug_from_key` correctness

**Inputs / expected outputs**:
- `"sdlc-run:my-feature"` → `"my-feature"`
- `"milestone-run-wave:v15-alpha"` → `"v15-alpha"`
- `"sdlc-run:feature-with-dashes"` → `"feature-with-dashes"`
- `"no-colon"` → `"unknown"`
- `""` → `"unknown"`
**Pass criteria**: All 5 cases produce correct output.

### TC-11: Build and lint pass

**Action**: `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`.
**Assert**: Zero test failures, zero clippy warnings.
**Pass criteria**: Clean build.

### TC-12: Smoke test — `sdlc merge` on a test feature

**Setup**: Create a test feature, advance it to merge phase (can use `sdlc feature transition` for setup).
**Action**: Run `sdlc merge <test-slug>`.
**Assert**: `.sdlc/changelog.yaml` exists; contains a `feature_merged` entry with `slug: <test-slug>`.
**Pass criteria**: CLI integration confirmed.

### TC-13: `SseMessage::ChangelogUpdated` variant exists and compiles

**Assert**: The `SseMessage` enum in `state.rs` has the `ChangelogUpdated` variant. The server compiles with it present.
**Pass criteria**: No compile error; clippy clean.

## Exit criteria

All TC-1 through TC-13 pass. Zero `unwrap()` in `event_log.rs` production code.

## Out of scope

- REST API endpoint for changelog query (changelog-api feature)
- UI rendering (changelog-dashboard-banner feature)
- Performance benchmarks for large files (tail-read optimization is a future task)
