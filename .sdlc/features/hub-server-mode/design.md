# Design: Hub Server Mode

## Architecture Overview

Hub mode is a new operating mode for `sdlc-server`. When activated, the server replaces
project-specific routes with hub-specific routes: heartbeat receiver, project registry, and
an SSE stream for live updates.

```
Client (project instance)          Hub Server
  every 30s                         :9999
  POST /api/hub/heartbeat  ───────► HubRegistry (in-memory HashMap)
                                      │
                                      ├─ sweep task (every 15s)
                                      │   marks offline / removes stale
                                      │   emits SSE events
                                      │
                                      ├─ persistence: ~/.sdlc/hub-state.yaml
                                      │   (written on every heartbeat + sweep)
                                      │
                                      └─ GET /api/hub/events (SSE)
                                             │
                                     Browser (hub UI)
```

## New Files

### `crates/sdlc-server/src/hub.rs`

Core hub data layer. Contains:

```rust
/// Heartbeat payload sent by project instances every 30s.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HeartbeatPayload {
    pub name: String,
    pub url: String,
    pub active_milestone: Option<String>,
    pub feature_count: Option<u32>,
    pub agent_running: Option<bool>,
}

/// One entry in the hub registry — derived from the most recent heartbeat.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub url: String,
    pub active_milestone: Option<String>,
    pub feature_count: Option<u32>,
    pub agent_running: Option<bool>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub status: ProjectStatus,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Online,   // last_seen < 30s
    Stale,    // 30s <= last_seen < 90s
    Offline,  // 90s <= last_seen < 5min
    // entries >5min are removed entirely
}

/// Thread-safe hub registry held by AppState.
pub struct HubRegistry {
    pub projects: std::collections::HashMap<String, ProjectEntry>,
    pub event_tx: tokio::sync::broadcast::Sender<HubSseMessage>,
    pub persist_path: std::path::PathBuf,
}

#[derive(Clone, Debug)]
pub enum HubSseMessage {
    ProjectUpdated(ProjectEntry),
    ProjectRemoved { url: String },
}
```

Key functions:
- `HubRegistry::new(persist_path) -> Self` — loads saved state (all entries start offline)
- `HubRegistry::apply_heartbeat(&mut self, payload) -> ProjectEntry` — upserts entry, updates `last_seen`, emits `ProjectUpdated`
- `HubRegistry::sweep(&mut self)` — marks stale/offline, removes >5min, emits `ProjectRemoved`
- `HubRegistry::persist(&self)` — writes `~/.sdlc/hub-state.yaml` via `atomic_write`
- `HubRegistry::projects_sorted(&self) -> Vec<ProjectEntry>` — sorted by `last_seen` descending

### `crates/sdlc-server/src/routes/hub.rs`

Three route handlers:

```rust
// POST /api/hub/heartbeat
pub async fn heartbeat(
    State(app): State<AppState>,
    Json(payload): Json<HeartbeatPayload>,
) -> impl IntoResponse

// GET /api/hub/projects
pub async fn list_projects(
    State(app): State<AppState>,
) -> impl IntoResponse

// GET /api/hub/events
pub async fn hub_sse_events(
    State(app): State<AppState>,
) -> impl IntoResponse
```

Both `heartbeat` and `list_projects` require `app.hub_registry` to be `Some(...)`.
If called in project mode (None), they return `503 Service Unavailable`.

## Changes to AppState (`state.rs`)

Add one new field:

```rust
/// Hub registry — Some(...) in hub mode, None in project mode.
pub hub_registry: Option<Arc<tokio::sync::Mutex<HubRegistry>>>,
```

Populated in `build_base_state` when hub mode is active:
- Hub mode: `hub_registry = Some(Arc::new(Mutex::new(HubRegistry::new(persist_path))))`
- Project mode: `hub_registry = None`

The sweep task is spawned in `new_with_port` alongside the existing file-watcher tasks,
guarded by `if hub_registry.is_some()`.

## Changes to `lib.rs`

Add hub mode flag to `build_router` / `serve` / `serve_on`:

```rust
pub fn build_router(root: PathBuf, port: u16, hub_mode: bool) -> Router { ... }
pub async fn serve(root: PathBuf, port: u16, open_browser: bool, hub_mode: bool, ...) -> Result<()>
pub async fn serve_on(root: PathBuf, listener: TcpListener, open_browser: bool,
                      hub_mode: bool, ...) -> Result<()>
```

In `build_router_from_state`, hub routes are always registered (they check `hub_registry`
internally). No conditional route registration needed — simpler and no route ordering hazards.

## Changes to CLI (`cmd/ui.rs`)

Add `--hub` flag to `sdlc ui start`:

```
--hub    Start in hub mode (project navigator, no project required)
```

Threads `hub_mode: bool` through `run_start` → `serve_on`.

In hub mode, skip the `.sdlc/config.yaml` load and use `"hub"` as the project name
for the UI registry record.

## Persistence Path

`~/.sdlc/hub-state.yaml` — in the user's home directory, not the project root.
Rationale: hub mode has no project root; the hub is a user-level service.

The persist path is constructed at startup:
```rust
dirs::home_dir()
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join(".sdlc")
    .join("hub-state.yaml")
```

`sdlc-server/Cargo.toml` gains a `dirs` dependency for home dir resolution.

## SSE Protocol

Hub SSE follows the same pattern as `/api/events`. Events:

```
event: hub
data: {"type":"project_updated","project":{...}}

event: hub
data: {"type":"project_removed","url":"http://localhost:3001"}
```

The hub UI subscribes to `GET /api/hub/events` and updates cards in real time.

## Sweep Timing

| Threshold | Action |
|---|---|
| < 30s | `status = online` (green dot) |
| 30–90s | `status = stale` (yellow dot) |
| 90s–5min | `status = offline` (grey dot) |
| > 5min | Removed from registry entirely |

Sweep runs every 15 seconds. Both the sweep and heartbeat handler recompute `status` from
`last_seen` at the time of read/write, so status is always current in API responses.

## Error Handling

- Heartbeat with missing `name` or `url` → `422 Unprocessable Entity`.
- `hub_registry` is `None` (project mode) → `503 Service Unavailable`.
- Persistence failure is logged as a warning, never fatal — hub continues running.

## ASCII Flow

```
sdlc ui start --hub --port 9999
  │
  ├─ build_base_state(root="/", hub_mode=true)
  │   └─ HubRegistry::new("~/.sdlc/hub-state.yaml")
  │       └─ load saved entries (all status=offline)
  │
  ├─ new_with_port (spawns sweep task every 15s)
  │
  └─ build_hub_routes added to router
      POST /api/hub/heartbeat
      GET  /api/hub/projects
      GET  /api/hub/events

Project instance:
  every 30s → POST /api/hub/heartbeat {name, url, ...}
                └─ HubRegistry::apply_heartbeat(payload)
                    ├─ upsert entry, last_seen = now, status = online
                    ├─ persist to ~/.sdlc/hub-state.yaml
                    └─ broadcast HubSseMessage::ProjectUpdated

Sweep task (every 15s):
  └─ HubRegistry::sweep()
      ├─ recompute status for all entries
      ├─ remove entries last_seen > 5min
      ├─ if changed: persist + broadcast ProjectRemoved
      └─ sleep 15s
```
