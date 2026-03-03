# Tasks: changelog-core

## Implementation task breakdown

The feature already has 10 tasks defined in the manifest (T1–T10). This document maps them to the implementation order and provides precise guidance for each.

### T1 — Create `crates/sdlc-core/src/event_log.rs`

Create the core module with:
- `EventKind` enum with 7 snake_case variants (serde rename_all snake_case)
- `ChangeEvent` struct: `kind`, `slug`, `timestamp: DateTime<Utc>`, `details: Option<serde_yaml::Value>`
- `append_event(root: &Path, event: &ChangeEvent) -> Result<()>` — uses `io::append_text` to append a YAML list entry
- `query_events(root, since, limit) -> Result<Vec<ChangeEvent>>` — reads full file, returns newest-first
- Add `pub const CHANGELOG_FILE` and `pub fn changelog_path` to `paths.rs`
- Add `pub mod event_log;` to `lib.rs`

### T2 — Emit `feature_merged` in `merge.rs`

After `state.save(root)` succeeds, call `event_log::append_event`. Non-fatal: `eprintln!` warn on error.

### T3 — Emit `review_approved`, `audit_approved`, `qa_approved` in `artifact.rs`

In `approve()`, after `feature.save(root)` succeeds, match artifact type to emit the appropriate `EventKind`. Non-fatal.

### T4 — Emit `run_failed` in `runs.rs`

After `RunFinished` SSE is sent, if `status == "failed"`, parse slug from key and emit `RunFailed`. Use `tokio::task::spawn_blocking` for the file write. Non-fatal.

### T5 — Emit `milestone_wave_completed` in `runs.rs`

After `RunFinished` SSE, if key starts with `"milestone-run-wave:"` and `status == "completed"`, emit `MilestoneWaveCompleted`. Add `extract_slug_from_key()` helper.

### T6 — Emit `feature_phase_advanced` in `classifier.rs`

In `try_auto_transition` (or wherever phase transition is logged post-save in the CLI layer), emit `FeaturePhaseAdvanced` when the resulting phase is `Implementation` or later. The implementation should be in `crates/sdlc-core/src/classifier.rs`'s `try_auto_transition` which has access to `root` and the resulting phase. Non-fatal.

### T7 — Add `SseMessage::ChangelogUpdated` and mtime watcher

In `crates/sdlc-server/src/state.rs`: add `ChangelogUpdated` variant to `SseMessage`.
In `crates/sdlc-server/src/main.rs`: add mtime watcher for `.sdlc/changelog.yaml`, emit `SseMessage::ChangelogUpdated` on change.

### T8 — Unit tests

In the `#[cfg(test)]` block of `event_log.rs`:
- `append_query_roundtrip`: append 3 events, query_events returns 3, newest-first
- `query_since_filter`: append t0 and t1 events, query since t0+1ms returns only t1
- `query_limit`: append 5 events, limit=2 returns newest 2
- `empty_file_ok`: write empty file, query returns Ok(vec![])
- `missing_file_ok`: no file, query returns Ok(vec![])
- `all_event_kinds_roundtrip`: serialize/deserialize each of the 7 EventKind variants

### T9 — [user-gap] Slug extraction from run key

Implement `extract_slug_from_key(key: &str) -> String` in `runs.rs`:
- `"sdlc-run:my-feature"` → `"my-feature"`
- `"milestone-run-wave:v15"` → `"v15"`
- anything without `:` → `"unknown"`
This is used by T4 and T5.

### T10 — [user-gap] `query_events` tail-read behavior

`query_events` reads from the end of the file safely. Implementation: read full file and parse as YAML sequence (initial impl). Document clearly with a code comment that tail-read optimization (seek from end) can replace this if performance becomes a concern at >100K events.

## Implementation order

T1 → T8 (core module + tests) → T9 (slug helper) → T2, T3 (CLI emit) → T6 (classifier emit) → T4, T5 (server emit) → T7 (SSE variant + watcher)

All tasks must pass `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings` before review.
