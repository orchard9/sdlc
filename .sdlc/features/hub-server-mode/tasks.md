# Tasks: Hub Server Mode

## Task List

### T1: Add `dirs` dependency to sdlc-server
Add `dirs = "5"` to `crates/sdlc-server/Cargo.toml` for home directory resolution.

### T2: Implement `hub.rs` — HubRegistry, ProjectEntry, sweep
Create `crates/sdlc-server/src/hub.rs` with:
- `HeartbeatPayload` struct (serde Deserialize)
- `ProjectEntry` struct (serde Serialize/Deserialize)
- `ProjectStatus` enum (`online`, `stale`, `offline`)
- `HubSseMessage` enum (`ProjectUpdated`, `ProjectRemoved`)
- `HubRegistry` struct with `projects: HashMap<String, ProjectEntry>`, `event_tx: broadcast::Sender<HubSseMessage>`, `persist_path: PathBuf`
- `HubRegistry::new(persist_path) -> Self` — loads saved state, marks all offline
- `HubRegistry::apply_heartbeat(&mut self, payload) -> ProjectEntry` — upsert, emit event
- `HubRegistry::sweep(&mut self)` — mark stale/offline/remove, emit events
- `HubRegistry::persist(&self)` — atomic_write to persist_path
- `HubRegistry::projects_sorted(&self) -> Vec<ProjectEntry>` — by last_seen desc

### T3: Implement `routes/hub.rs` — heartbeat, list, SSE handlers
Create `crates/sdlc-server/src/routes/hub.rs` with:
- `POST /api/hub/heartbeat` handler: deserialize payload, lock registry, call apply_heartbeat, return 200
- `GET /api/hub/projects` handler: lock registry, return projects_sorted as JSON
- `GET /api/hub/events` handler: subscribe to hub event_tx, stream HubSseMessage as SSE events
- 503 fallback when hub_registry is None

### T4: Add `hub_registry` field to AppState
In `crates/sdlc-server/src/state.rs`:
- Add `pub hub_registry: Option<Arc<Mutex<HubRegistry>>>` to `AppState`
- In `build_base_state`, accept `hub_mode: bool` parameter; when true, construct `HubRegistry::new(home_hub_path)` and wrap in `Arc<Mutex<...>>`
- Add sweep task in `new_with_port` when hub_registry is Some

### T5: Register hub routes and add hub mode to lib.rs
In `crates/sdlc-server/src/lib.rs`:
- Add `hub_mode: bool` param to `build_router`, `serve`, `serve_on`
- Thread `hub_mode` into `AppState` construction
- Register hub routes: `POST /api/hub/heartbeat`, `GET /api/hub/projects`, `GET /api/hub/events`
- Export `pub mod hub` in `lib.rs`

### T6: Add `--hub` flag to CLI (`cmd/ui.rs`)
In `crates/sdlc-cli/src/cmd/ui.rs`:
- Add `--hub` flag to `UiSubcommand::Start` and top-level args
- In hub mode, skip `.sdlc/config.yaml` load; use `"hub"` as project name
- Pass `hub_mode` through `run_start` → `serve_on`

### T7: Wire `hub` module into server lib.rs
In `crates/sdlc-server/src/lib.rs`:
- Add `pub mod hub;`
In `crates/sdlc-server/src/routes/mod.rs`:
- Add `pub mod hub;`

### T8: Add unit tests for HubRegistry
In `crates/sdlc-server/src/hub.rs`:
- Test `apply_heartbeat` creates entry with `status=online`
- Test `sweep` marks entry offline after threshold
- Test `sweep` removes entry after 5min
- Test `new()` loads persisted state and marks all offline
