# QA Plan: backlog-core

## Verification Approach

All QA is automated via `cargo test`. No manual steps required.

## Test Command

```bash
SDLC_NO_NPM=1 cargo test -p sdlc-core backlog
```

## Test Cases

| # | Test Name | Verifies |
|---|---|---|
| 1 | `add_creates_item_with_b1_id` | First add returns B1 |
| 2 | `add_sequential_ids` | Second add returns B2 |
| 3 | `add_persists_all_fields` | description, evidence, source_feature stored correctly |
| 4 | `list_unfiltered_returns_all` | All statuses returned when no filter |
| 5 | `list_open_status_filter` | Only Open items returned |
| 6 | `list_by_source_feature` | Only items from specified feature |
| 7 | `list_combined_filters` | Open AND source_feature intersection |
| 8 | `get_existing_returns_item` | Correct item returned by ID |
| 9 | `get_missing_id_errors` | Returns `BacklogItemNotFound` |
| 10 | `park_sets_status_and_reason` | status→Parked, park_reason set, updated_at refreshed |
| 11 | `park_requires_nonempty_reason` | Empty reason string → error |
| 12 | `park_promoted_item_errors` | Cannot park an already-promoted item |
| 13 | `mark_promoted_sets_slug` | status→Promoted, promoted_to set |
| 14 | `mark_promoted_from_parked_ok` | Parked items can be promoted |
| 15 | `round_trip_serialization` | load(save(store)) == store |

## Pass Criteria

All 15 tests pass with zero warnings (`cargo clippy -p sdlc-core -- -D warnings`).
