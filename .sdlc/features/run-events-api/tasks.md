# Tasks: run-events-api

- [ ] Add `redb = { workspace = true }` to `crates/sdlc-server/Cargo.toml`
- [ ] Create `crates/sdlc-server/src/telemetry.rs` — `TelemetryStore` struct with `open()`, `append_raw()`, `events_for_run()`, `summary_for_run()`; composite key `(run_id, seq)`; prefix-range scan using `next_string()`
- [ ] Add `pub telemetry: Option<Arc<TelemetryStore>>` to `AppState` in `state.rs`; initialize from `.sdlc/telemetry.redb` in `AppState::new_with_port()`
- [ ] Wire `store.append_raw()` non-blocking call in `spawn_agent_run()` after `accumulated_events.push(event.clone())`
- [ ] Create `crates/sdlc-server/src/routes/telemetry.rs` — `get_run_telemetry` handler (`GET /api/runs/:id/telemetry`) and `get_run_telemetry_summary` handler (`GET /api/runs/:id/telemetry/summary`)
- [ ] Register telemetry routes in the router; add `.sdlc/telemetry.redb` to `.gitignore`
