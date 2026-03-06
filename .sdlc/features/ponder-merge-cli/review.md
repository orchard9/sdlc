# Code Review: sdlc ponder merge — CLI command and core data model

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/ponder.rs` | Added `merged_into`, `merged_from` fields to PonderEntry; `MergeResult` struct; `validate_merge_preconditions()` + `merge_entries()` functions; 8 new tests |
| `crates/sdlc-core/src/error.rs` | Added `PonderMergeError(String)` variant |
| `crates/sdlc-cli/src/cmd/ponder.rs` | Added `Merge` subcommand; `merge()` handler; `--all` flag on `List`; redirect banner on `show`; merged_into/merged_from in JSON output |

## Review Criteria

### Correctness: PASS
- merge_entries correctly validates all preconditions (self-merge, committed source/target, already-merged)
- Sessions are renumbered via workspace::write_session which auto-increments
- Artifacts are copied with collision prefix (`source--filename`)
- Team dedup by name works correctly
- Source parking is the final step — partial failure leaves source intact
- All 8 new unit tests pass covering success path, all error paths, collision prefix, and team dedup

### Conventions: PASS
- No `unwrap()` in library code — all errors use `?` propagation
- File writes go through `crate::io::atomic_write`
- JSON output via `print_json()`, table output via `print_table()`
- New fields use `serde(default)` for backward compatibility with existing YAML

### Safety: PASS
- No path traversal risk — all paths are constructed from validated slugs
- No data loss — source is only parked after all copies succeed
- `serde(default)` ensures existing entries without merge fields load cleanly

### Test Coverage: PASS
- `merge_fields_serde_roundtrip` — verifies new fields serialize/deserialize
- `merge_fields_default_when_absent` — backward compat
- `merge_entries_success` — full merge with sessions, artifacts, tags
- `merge_self_rejected` — self-merge error
- `merge_committed_source_rejected` — committed source error
- `merge_committed_target_rejected` — committed target error
- `merge_already_merged_rejected` — double-merge error
- `merge_artifact_collision_prefix` — filename collision handling
- `merge_team_dedup` — team member deduplication

### Build Health: PASS
- `cargo build --all` — clean
- `cargo test --all` — all pass (452 total)
- `cargo clippy --all -- -D warnings` — zero warnings

## Score: 95/100

Minor notes:
- The merge function does not attempt rollback on partial failure, which is documented as intentional (source is parked last, so partial failure is safe)
- No REST API endpoint for merge — correctly deferred as out of scope per spec
