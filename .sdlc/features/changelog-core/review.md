# Review: changelog-core

## Summary

Code review of the `changelog-core` implementation: the append-only event log module, emission call sites, SSE integration, and tests.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/event_log.rs` | Core module (pre-existing, reviewed for spec compliance) |
| `crates/sdlc-core/src/lib.rs` | `pub mod event_log;` (pre-existing) |
| `crates/sdlc-core/src/classifier.rs` | Added `feature_phase_advanced` emission in `try_auto_transition` |
| `crates/sdlc-cli/src/cmd/merge.rs` | Added `feature_merged` emission after successful merge |
| `crates/sdlc-cli/src/cmd/artifact.rs` | Added `review_approved`, `audit_approved`, `qa_approved` emissions on approve |
| `crates/sdlc-server/src/routes/runs.rs` | Added `run_failed`, `milestone_wave_completed` emissions + `extract_slug_from_key` helper |
| `crates/sdlc-server/src/state.rs` | Added `SseMessage::ChangelogUpdated` variant + mtime watcher for `changelog.yaml` |

## Correctness

### Event log module (`event_log.rs`)

The existing implementation uses a read-then-write (atomic rewrite) strategy for `append_event`, not a pure append. This differs from the spec's `io::append_text` intent but is functionally equivalent: each call reads the current list, pushes the new event, serializes the full list, and atomically writes. This approach guarantees valid YAML at all times and is correct for the current use case.

The `id` field (sequential `ev-0001`, `ev-0002`...) is present and adds value for external consumers — not in the original spec but a net positive. The `slug` field is `Option<String>` rather than required `String` — this is slightly more permissive than the spec but consistent with the rest of the codebase's patterns.

All 7 `EventKind` variants are present and correctly serialize/deserialize to snake_case via `#[serde(rename_all = "snake_case")]`.

### Emission call sites

**`merge.rs`**: Emission is after `state.save(root)`. Correct placement — the feature is already released when the event is recorded. Error is suppressed with `eprintln!`. Correct.

**`artifact.rs`**: Emission is after `feature.save(root)` and before `try_auto_transition`. The three artifact types (`Review`, `Audit`, `QaResults`) are matched correctly. Error is suppressed with `eprintln!`. Correct.

**`classifier.rs`**: Emission happens inside `try_auto_transition` after `feature.save(root).is_ok()`. The `matches!` arm correctly covers `Implementation`, `Review`, `Audit`, `Qa`, `Merge`, `Released`. Error is ignored via `let _ = ...`. Correct. The `serde_json::json!` for details is used to produce the `serde_yaml::Value` implicitly — this works because `serde_json::Value` is de/serializable and serde_yaml will accept it.

**`runs.rs`**: The `extract_slug_from_key` helper correctly splits on `:` and returns `"unknown"` for keys without colons. The `spawn_blocking` wrapper is correct — file I/O should not run on the async executor. The `run_failed` and `milestone_wave_completed` conditions are exclusive and correct. `.await.ok()` suppresses any panic in the blocking task. Correct.

**`state.rs`**: `SseMessage::ChangelogUpdated` is correctly added as a unit variant. The mtime watcher follows the exact same 800ms polling pattern as all other watchers. Correct.

### Non-fatal emission

All 5 call sites correctly treat changelog write failures as non-fatal. CLI call sites use `eprintln!("warn: ...")`. `classifier.rs` and `runs.rs` silently discard errors. This is correct per spec — the changelog is observability data, not critical-path state.

## Test coverage

The 6 existing unit tests in `event_log.rs` cover:
- Empty file and missing file return Ok(vec![])
- Append/query round-trip
- Limit enforcement
- `since` filter (both exclude-old and include-new cases)
- Sequential ID generation

All tests pass: `SDLC_NO_NPM=1 cargo test --all` shows 358+ passed, 0 failed.
Clippy: `cargo clippy --all -- -D warnings` clean.

## Issues found

### Minor: No `unwrap()` in library code — verified

No `unwrap()` calls in `event_log.rs` production code paths. The test code uses `unwrap()` which is appropriate in tests.

### Minor: `Option<String>` slug vs required `String`

The spec required `slug: String` (non-optional). The existing implementation uses `slug: Option<String>`. The emission call sites that use `Some(slug.to_string())` are correct, but future callers could accidentally pass `None`. This is a minor interface fidelity gap — documented and accepted.

**Action**: Create a follow-up task.

### Minor: `serde_json::json!` used for YAML metadata

The `classifier.rs` emission uses `serde_json::json!({ "phase": ... })` as the `metadata` value. Since the function signature expects `serde_json::Value`, this is correct and will serialize cleanly to YAML. No issue.

### Minor: Watcher polls every 800ms

The changelog watcher uses 800ms polling, same as all other watchers. This means a changelog event could take up to 800ms to appear in the SSE stream. This is acceptable for the current use case (activity feed, not real-time alerting).

## Verdict

Implementation is correct, complete, and consistent with the existing codebase patterns. All 5 emission call sites are wired. The SSE variant and mtime watcher are in place. Tests pass, clippy is clean.

One follow-up task added for the `slug: Option<String>` vs required `String` discrepancy.
