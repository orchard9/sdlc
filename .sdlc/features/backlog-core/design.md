# Design: backlog-core

## Architecture

This is a pure data layer addition to `sdlc-core`. No decision logic, no heuristics — just
struct definitions, serialization, and atomic file I/O. All promotion logic (feature creation,
milestone linking, source_feature inference) belongs in `sdlc-cli`.

## File Structure

```
crates/sdlc-core/src/
  backlog.rs          ← new module
  paths.rs            ← +BACKLOG_FILE const, +backlog_path()
  error.rs            ← +BacklogItemNotFound(String)
  lib.rs              ← +pub mod backlog
```

## Module Design: backlog.rs

```
BacklogKind (enum)      concern | idea | debt
BacklogStatus (enum)    open | parked | promoted
BacklogItem (struct)    all fields, optional via Option<T>
BacklogStore (struct)   { items: Vec<BacklogItem> }
  impl BacklogStore
    ::load(root)          → Result<Self>      reads + deserializes backlog.yaml
    ::save(&self, root)   → Result<()>        atomic_write
    ::add(root, ...)      → Result<BacklogItem>
    ::list(root, ...)     → Result<Vec<BacklogItem>>
    ::get(root, id)       → Result<BacklogItem>
    ::park(root, id, reason) → Result<BacklogItem>
    ::mark_promoted(root, id, slug) → Result<BacklogItem>
    fn next_id(items)     → String            private
```

All mutating methods load → mutate → save atomically (single load/save per call).

## ID Scheme

Sequential B-prefix integers. `next_id` scans `items`, strips "B" prefix, parses as `u32`,
takes the max, returns `format!("B{}", max + 1)`. If items is empty, returns `"B1"`.
IDs are never recycled.

## park() Semantics

- Validates item exists and is `Open`. If `Promoted`, error: cannot park a promoted item.
- Validates `park_reason` is non-empty. If empty, returns `SdlcError::BacklogItemNotFound`
  is wrong — needs a new error variant approach. Use a descriptive message in a suitable
  existing error or inline validation.
- Sets `status = Parked`, `park_reason = Some(reason)`, `updated_at = now`.
- Saves and returns the updated item.

Actually, for validation of empty park_reason, we'll use `SdlcError::Io` with a
descriptive message is not ideal. Better approach: add a validation check and return
a custom error string via an existing variant. The cleanest option is to treat it as
a constraint violation — return `Err(SdlcError::InvalidTransition { from, to, reason })`
with a helpful reason string. This reuses existing infrastructure.

## mark_promoted() Semantics

- Validates item exists and is `Open` (can also promote a parked item — captures intent).
- Sets `status = Promoted`, `promoted_to = Some(slug)`, `updated_at = now`.
- Does NOT create a feature — that's the CLI's job.
- Saves and returns the updated item.

## list() Filtering

Both filters are independent and composed with AND:
- `status_filter: None` → return all statuses (callers wanting open-only pass `Some(BacklogStatus::Open)`)
- `source_feature: None` → return all features
- Results are sorted by `created_at` ascending (oldest first is stable for tests).

## Serialization Conventions

Matches the rest of sdlc-core:
- `#[serde(rename_all = "snake_case")]` on all enums
- `#[serde(skip_serializing_if = "Option::is_none")]` on all `Option<T>` fields
- `chrono::DateTime<Utc>` for timestamps, serialized as ISO 8601 strings
- Top-level: `{ items: [...] }`; empty file → `{ items: [] }`

## Test Strategy

All tests use `tempfile::TempDir` for isolated `.sdlc/` directories, matching the
existing test pattern in `advisory.rs` and `escalation.rs`.

Test cases:
1. `add_creates_item_with_b1_id` — first add returns B1
2. `add_sequential_ids` — second add returns B2
3. `list_unfiltered` — returns all items
4. `list_open_only` — returns only open items
5. `list_by_source_feature` — returns only items from that feature
6. `list_combined_filters` — open AND source_feature
7. `get_existing` — returns correct item
8. `get_missing_id` — returns BacklogItemNotFound
9. `park_sets_status_and_reason` — status becomes parked, park_reason set
10. `park_requires_nonempty_reason` — empty reason → error
11. `park_already_promoted_errors` — cannot park a promoted item
12. `mark_promoted_sets_status` — status becomes promoted, promoted_to set
13. `mark_promoted_from_parked_ok` — parked items can be promoted
14. `round_trip_serialization` — write + read produces identical struct
