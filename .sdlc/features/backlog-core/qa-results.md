# QA Results: backlog-core

## Test Execution

```
SDLC_NO_NPM=1 cargo test -p sdlc-core backlog
```

## Results

| Test | Result |
|---|---|
| `add_creates_item_with_b1_id` | ✓ pass |
| `add_sequential_ids` | ✓ pass |
| `add_persists_all_fields` | ✓ pass |
| `list_unfiltered_returns_all` | ✓ pass |
| `list_open_status_filter` | ✓ pass |
| `list_by_source_feature` | ✓ pass |
| `list_combined_filters` | ✓ pass |
| `get_existing_returns_item` | ✓ pass |
| `get_missing_id_errors` | ✓ pass |
| `park_sets_status_and_reason` | ✓ pass |
| `park_requires_nonempty_reason` | ✓ pass |
| `park_whitespace_only_reason_errors` | ✓ pass |
| `park_promoted_item_errors` | ✓ pass |
| `mark_promoted_sets_slug` | ✓ pass |
| `mark_promoted_from_parked_ok` | ✓ pass |
| `mark_promoted_already_promoted_errors` | ✓ pass |
| `round_trip_serialization` | ✓ pass |
| `load_absent_file_returns_empty` | ✓ pass |

**18/18 passed. 0 failed.**

## Lint

```
cargo clippy -p sdlc-core -- -D warnings → 0 warnings
```

## Verdict: PASS
