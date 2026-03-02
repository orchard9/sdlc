# Code Review: knowledge-core-data

## Summary

Implementation is complete and correct. All 19 tests pass. Clippy is clean.

## Files reviewed

- `crates/sdlc-core/src/knowledge.rs` (new, ~580 lines + 100 lines tests)
- `crates/sdlc-core/src/error.rs` (+4 variants)
- `crates/sdlc-core/src/paths.rs` (+3 constants, +5 helpers, +1 test)
- `crates/sdlc-core/src/lib.rs` (+1 line)

## Checklist

**Correctness**
- [x] Slug-only directory naming: `knowledge_dir(root, slug)` uses only the slug — code never enters path
- [x] `reclassify_does_not_rename_dir` test explicitly proves reclassification safety
- [x] `validate_code` uses OnceLock<Regex> matching the exact pattern from the spec
- [x] `list` correctly skips top-level files (catalog.yaml, maintenance-log.yaml) by checking `is_file()`
- [x] `full_text_search` returns empty vec (not error) when knowledge dir absent
- [x] Graceful empty states: catalog, maintenance log, list all return empty on missing files

**Code quality**
- [x] No `unwrap()` in library code — all errors propagate via `?`
- [x] All writes use `crate::io::atomic_write`
- [x] serde attributes consistent with rest of codebase (`skip_serializing_if`, `default`)
- [x] `#[allow(clippy::too_many_arguments)]` used appropriately on `update` (8 params, correct for this data layer)

**Tests**
- [x] 18 knowledge tests + 1 paths test = 19 total
- [x] All edge cases covered: duplicates, empty base, reclassification, invalid codes
- [x] Uses `tempfile::TempDir` for isolation — no global state

**Consistency**
- [x] Pattern matches investigation.rs and ponder.rs (workspace delegation, serde style)
- [x] Module alphabetically placed in lib.rs (`investigation` → `io` → `knowledge` → `migrations`)

## Verdict

Approved. No issues found. Ready to transition to audit.
