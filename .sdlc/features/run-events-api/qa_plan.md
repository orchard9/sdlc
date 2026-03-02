# QA Plan: run-events-api

## Unit tests

1. `TelemetryStore::append_raw` then `events_for_run` — assert round-trip preserves all fields
2. Two concurrent run_ids — assert `events_for_run(run_a)` returns only run A events (prefix isolation)
3. `summary_for_run` — assert `tool_calls`, `tool_errors`, `tools_used` counts are correct for a known event sequence
4. Server restart simulation — open store, write events, drop store, re-open, assert events persist

## API tests

- `GET /api/runs/:id/telemetry` for a known run — assert 200 with correct `run_id` and non-empty `events` array
- `GET /api/runs/:id/telemetry/summary` — assert 200 with correct aggregated stats
- `GET /api/runs/nonexistent/telemetry` — assert 200 with empty `events` (not 404)

## Integration check

- Run a feature agent run; hit the telemetry API; assert the event count matches the events sidecar file count
