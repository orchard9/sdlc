# QA Results: activity-time-series

## Status: PASS WITH TASKS

## Automated checks

### QA-25: TypeScript compilation clean
- Command: `cd frontend && npx tsc --noEmit`
- Result: **PASSED** — zero errors

### File presence
All new files confirmed present:
- `frontend/src/components/runs/buildTimeSeries.ts` — exists
- `frontend/src/components/runs/ActivityTimeSeries.tsx` — exists
- `frontend/src/components/layout/RunCard.tsx` — updated
- `frontend/src/components/runs/RunActivityFeed.tsx` — updated
- `frontend/src/lib/types.ts` — updated

### Backend tests
- `cargo test --package sdlc-core` — **PASSED** (0 failures)
- `cargo test --package sdlc-server` — **SKIPPED** (pre-existing compile errors in server: `WebhookEvent` import + `Table::len` API change — unrelated to this feature, no Rust changes were made)

## Manual smoke tests

### QA-17, QA-18, QA-19 (chart in RunCard, fallback, live update)
- **DEFERRED** — requires `telemetry-wallclock-timestamps` to be implemented before `ts` fields appear in run events. Without timestamp data, all runs show the fallback message "Time breakdown not available (run predates timestamps)". The fallback renders correctly (verified by code inspection — `buildTimeSeries` returns `null` when no `ts` fields are present, and `ActivityTimeSeries` renders the fallback `<p>` element).
- The chart component and its integration in `RunCard` are syntactically and type-check correct.

### QA-23, QA-24 (RunActivityFeed regression, stop button)
- `RunActivityFeed` backward compatibility verified by code inspection:
  - When `events` prop is absent, `skipFetch = false` and `useRunTelemetry(runId, isRunning)` fires as before
  - When `events` prop is provided, `skipFetch = true` and no HTTP request is made
- Stop button logic in `RunCard` is unchanged (same `handleStop` callback, same `getStopDetails` function)

## Tracked tasks for post-`telemetry-wallclock-timestamps` verification

- T1 (tracked): Manual chart render verification (QA-17)
- T2 (tracked): Fallback for legacy runs (QA-18)
- T3 (tracked): Live update during active run (QA-19)
- T4 (tracked): Tooltip accuracy (QA-20)
- T5 (tracked): Responsive chart width (QA-21)

These tasks will be closed when `telemetry-wallclock-timestamps` ships and produces actual `ts` fields in run events.

## Summary

Core implementation is complete and type-safe. The feature degrades gracefully for all existing runs (shows fallback text). Full visual QA is dependent on the `telemetry-wallclock-timestamps` prerequisite feature.
