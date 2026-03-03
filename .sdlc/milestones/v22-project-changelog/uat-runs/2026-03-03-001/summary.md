# UAT Run — Project Changelog — what changed since you last looked
**Date:** 2026-03-03T03:32:00Z
**Verdict:** Failed
**Tests:** 1/7 scenarios fully testable
**Tasks created:** changelog-cli#T5, changelog-cli#T6, changelog-core#T13

## Hard Blocker

**localhost:7777 unreachable** — sdlc server is not running. Per UAT rules, server was not restarted.
This blocks UI scenarios 1–4 and API scenario 6 entirely.

## Results

| Scenario | Status | Notes |
|---|---|---|
| S1: First Visit (New Developer) | BLOCKED | Server not running |
| S2: Returning User With Changes | BLOCKED | Server not running |
| S3: Dismiss Persists Across SPA Navigation | BLOCKED | Server not running |
| S4: Failed Run Link Works | BLOCKED | Server not running |
| S5: CLI Changelog | FAIL | JSON field mismatch + wrong data source |
| S6: API Endpoint | BLOCKED | Server not running |
| S7: Event Log Integrity | PASS | changelog.yaml has correct event structure |

## Scenario 5 Failures

| Check | Classification | Resolution |
|---|---|---|
| `sdlc changelog` reads .runs/ not changelog.yaml — merges/approvals invisible | Code bug | Task changelog-cli#T5 created |
| `sdlc changelog --json` fields: `category`≠`kind`, `started_at`≠`timestamp`, missing `slug`/`meta` | Code bug | Task changelog-cli#T6 created |

## Scenario 7 Notes

- `changelog.yaml` contains 83 events (ev-0001 to ev-0083) ✓
- `feature_merged` events: 10 present with correct `id`, `kind`, `slug`, `timestamp` structure ✓
- `review_approved` events: 7 present with correct structure ✓
- Code paths confirmed: `merge.rs` and `artifact.rs` both call `event_log::append_event` ✓
- Minor: `ChangeEvent` missing `label` field (spec expects `slug`, `label`, `timestamp`) → Task changelog-core#T13

## Scenario 5 Notes

- `sdlc changelog` output verified: shows 20 agent run events with `▶` icon
- Icon categories are defined in code (⚠️ RunFailed, 🚀 FeatureMerged, ✅ Approval, 🔄 PhaseAdvanced) ✓
- BUT: `sdlc changelog` reads `.sdlc/.runs/*.json` (agent run records), NOT `changelog.yaml`
- Direct `sdlc merge` and `sdlc artifact approve` calls write to `changelog.yaml` but are invisible in CLI output
- JSON shape: `{ since, limit, total, events: [{id, category, icon, label, run_type, status, started_at, cost_usd?}] }`
- Spec expected: `[{id, kind, timestamp, label, slug, meta}]`
