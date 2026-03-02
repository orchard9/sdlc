# Code Review: backlog-core

## Summary

Added the `BacklogItem` primitive to `sdlc-core`: data model, CRUD operations, and
18 unit tests. Implementation follows all project conventions.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/backlog.rs` | Created — full module |
| `crates/sdlc-core/src/paths.rs` | Added `BACKLOG_FILE` const + `backlog_path()` |
| `crates/sdlc-core/src/error.rs` | Added `BacklogItemNotFound(String)` variant |
| `crates/sdlc-core/src/lib.rs` | Added `pub mod backlog` |

## Review Findings

### Correctness ✓

- `next_id` correctly handles empty store (returns B1) and non-empty (max+1). IDs never recycled.
- `park()` validates non-empty reason with `trim()` — catches whitespace-only strings.
- `park()` rejects already-promoted items with a clear error.
- `mark_promoted()` allows promoting from Parked state (correct per spec).
- `mark_promoted()` rejects double-promotion with the current `promoted_to` in the message.
- All mutating methods use load→mutate→save pattern with `io::atomic_write`. No partial writes.

### Conventions ✓

- No `unwrap()` in non-test code — all errors use `?` and `SdlcError`.
- All enums use `#[serde(rename_all = "snake_case")]`.
- All `Option<T>` fields use `#[serde(skip_serializing_if = "Option::is_none")]`.
- `Display` impls on enums match serde snake_case representation.

### Architecture ✓

- Rust layer is pure data: load/save/mutate only. No promotion logic (feature creation
  stays in CLI), no source_feature inference (stays in CLI). Clean boundary.
- `BacklogStore` wraps `Vec<BacklogItem>` under an `items` key — matches `advisory.yaml`
  and `escalations.yaml` pattern. Consistent with the rest of the system.

### Tests ✓

- 18 tests covering all 14 design doc cases plus extras (whitespace reason, double-promote).
- All use `tempfile::TempDir` for isolated `.sdlc/` directories.
- Tests verify both in-memory return values AND persistence (load after write).

### Test Results

```
running 18 tests ... test result: ok. 18 passed; 0 failed
cargo clippy -p sdlc-core -- -D warnings → zero warnings
```

## Issues Found

None. Implementation is complete and correct.
