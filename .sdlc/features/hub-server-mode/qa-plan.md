# QA Plan: Hub Server Mode

## Testing Strategy

Unit tests for `HubRegistry` cover the core state machine logic. Integration tests cover
the HTTP endpoints using `build_router_for_test`. Clippy enforces no `unwrap()` in library code.

## Unit Tests (`crates/sdlc-server/src/hub.rs`)

### HU-1: apply_heartbeat creates entry with status online
- Call `apply_heartbeat` with a valid payload.
- Assert entry exists in registry with `status = online`.
- Assert `last_seen` is within a few milliseconds of now.

### HU-2: apply_heartbeat updates existing entry
- Call `apply_heartbeat` twice with the same URL.
- Assert only one entry exists in registry.
- Assert `last_seen` is updated.

### HU-3: sweep marks entry stale after 30s
- Insert an entry with `last_seen = now - 45s`.
- Call `sweep()`.
- Assert `status = stale`.

### HU-4: sweep marks entry offline after 90s
- Insert an entry with `last_seen = now - 120s`.
- Call `sweep()`.
- Assert `status = offline`.

### HU-5: sweep removes entry after 5 minutes
- Insert an entry with `last_seen = now - 310s`.
- Call `sweep()`.
- Assert entry is no longer in registry.

### HU-6: new() loads persisted state and marks all offline
- Write a valid hub-state.yaml with one `status=online` entry.
- Call `HubRegistry::new(path)`.
- Assert entry is loaded with `status=offline`.

### HU-7: projects_sorted returns most-recent first
- Insert three entries with different `last_seen` timestamps.
- Assert `projects_sorted()` returns them newest-first.

## Integration Tests (`crates/sdlc-server/tests/`)

### HI-1: POST /api/hub/heartbeat returns 200 and registers project
```
POST /api/hub/heartbeat
{"name":"test","url":"http://localhost:3001","feature_count":2}
→ 200 {"registered":true}
GET /api/hub/projects → contains entry with name="test"
```

### HI-2: POST /api/hub/heartbeat with missing name returns 422
```
POST /api/hub/heartbeat {"url":"http://localhost:3001"}
→ 422
```

### HI-3: GET /api/hub/projects in project mode returns 503
- Build router with hub_mode=false.
- `GET /api/hub/projects` → 503.

### HI-4: GET /api/hub/events streams ProjectUpdated on heartbeat
- Subscribe to SSE stream.
- POST heartbeat.
- Assert SSE event with `type=project_updated` is received.

## Clippy / Build Gates

- `cargo clippy --all -- -D warnings` must pass with no new warnings.
- `SDLC_NO_NPM=1 cargo test --all` must pass.

## Manual Smoke Test

1. `sdlc ui start --hub --port 9999 --no-tunnel`
2. `curl -X POST http://localhost:9999/api/hub/heartbeat -H 'Content-Type: application/json' -d '{"name":"test","url":"http://localhost:3001"}'`
3. `curl http://localhost:9999/api/hub/projects` → one entry, status=online
4. Wait 95 seconds without sending heartbeats
5. `curl http://localhost:9999/api/hub/projects` → entry status=offline
6. Wait 5 more minutes
7. `curl http://localhost:9999/api/hub/projects` → empty array
8. Restart server, verify `~/.sdlc/hub-state.yaml` was written and loaded on restart
