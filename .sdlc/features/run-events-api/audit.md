# Audit: run-events-api

## Security

- No user-supplied data is interpreted as code — run IDs from path parameters are used only as redb key prefixes
- The prefix-range scan uses `next_string()` to compute an exclusive upper bound; the implementation correctly handles the 0xFF edge case
- Telemetry is best-effort — failure silently drops with `let _ =`, never propagates to the agent execution path
- No sensitive data (credentials, secrets) is written to the telemetry store — events are the same JSON that was already accumulated in-memory and written to the `.events.json` sidecar

## Correctness

- redb write transactions are serialized by the database — no additional synchronization needed for concurrent runs
- Per-run sequence counters are initialized from the DB on first write — correct restart-safe behavior
- `events_for_run` returns events in sequence order due to redb's key ordering guarantee on the composite key `(run_id, seq)`
- `summary_for_run` correctly matches event types to the JSON structure produced by `message_to_event()` in `runs.rs`

## Performance

- Each event write is a single redb write transaction (~100-500 bytes per event)
- `spawn_blocking` keeps event writes off the async event loop — correct
- Reads are concurrent (redb read transactions do not block write transactions)
- Per-run counter initialization is O(k log n) on first access — bounded to the events for a single run

## Persistence

- Database file path: `.sdlc/telemetry.redb` — correctly gitignored
- Server restart recovery: counters are re-initialized from DB on first write — events are not lost
- Graceful degradation: `Option<Arc<TelemetryStore>>` ensures server starts even if `.sdlc/` is not writable

## Dependencies

- `redb = "2"` was already in `[workspace.dependencies]` — no new transitive dependencies
- `anyhow` context added to redb errors — satisfies clippy `result-large-err` without boxing

## Verdict: Approved

The feature is production-ready for a local dev tool. All risks identified in the spec (file growth, redb serialized writes) are acceptable for the expected load.
