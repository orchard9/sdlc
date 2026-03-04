# QA Plan: hub-heartbeat-client

## Automated Tests

### Unit: `spawn_heartbeat_task` returns None when SDLC_HUB_URL unset
- Ensure env var is absent (or explicitly removed).
- Call `spawn_heartbeat_task` with a test AppState.
- Assert return value is `None`.
- Verify no background task was spawned.

### Unit: `build_payload` returns correct name
- Create a temp dir with a known name as project root.
- Assert payload `name` equals the directory basename.

### Unit: `build_payload` returns agent_running = false when no active runs
- Empty `agent_runs` map → `agent_running` should be `false`.

### Unit: `build_payload` returns feature_count = 0 for empty features dir
- Create `.sdlc/features/` with no subdirectories → `feature_count` Some(0).
- Missing features dir → `feature_count` None.

### Clippy / Lint
- `cargo clippy --all -- -D warnings` must pass with zero warnings.

### Full test suite
- `SDLC_NO_NPM=1 cargo test --all` must pass.

## Manual Smoke Test (against a running hub)

1. Start hub server: `sdlc serve --hub`
2. Set env vars: `SDLC_HUB_URL=http://localhost:9999` and `SDLC_BASE_URL=http://localhost:8080`
3. Start project server: `sdlc serve`
4. After 30s, call `GET /api/hub/projects` on hub server.
5. Assert project entry appears with correct name, url, feature_count, agent_running.

## Negative Cases

| Scenario | Expected |
|---|---|
| Hub server unreachable | warn! logged, loop continues, server remains responsive |
| Hub returns 503 | warn! logged, loop continues |
| Hub URL is malformed | reqwest error → warn! logged, loop continues |
| SDLC_HUB_URL unset | No task spawned, no log output |
| SDLC_BASE_URL unset | `http://localhost:{port}` used as fallback |
