# Spec: Hub Server Mode

## Overview

Add a hub mode to `sdlc-server` that acts as a project navigator. When started with `--hub`
(or from a directory without `.sdlc/`), the server renders a hub UI instead of the project
workspace and maintains an in-memory registry of connected project instances.

## Problem Statement

Running multiple sdlc project instances creates context-switching friction — opening each
project's UI requires knowing its port. A hub server consolidates visibility: all active
project instances report their status via heartbeat, and the hub presents a unified navigator.

## Functional Requirements

### 1. Hub Mode Detection

- Hub mode activates when `sdlc serve --hub` flag is passed.
- Hub mode also activates automatically when the working directory contains no `.sdlc/` folder.
- In hub mode the server must NOT attempt to load project config, feature state, or any
  `.sdlc/` artifacts — those paths don't exist.

### 2. POST /api/hub/heartbeat

Accepts a JSON payload from project instances every 30 seconds:

```json
{
  "name": "payments-api",
  "url": "http://localhost:3001",
  "active_milestone": "v12-checkout-flow",
  "feature_count": 3,
  "agent_running": false
}
```

- All fields except `name` and `url` are optional.
- First heartbeat from a new `url` is a registration event.
- Updates `last_seen` timestamp on each subsequent heartbeat.
- Returns `200 OK` with `{ "registered": true }`.

### 3. GET /api/hub/projects

Returns the current registry as a JSON array:

```json
[
  {
    "name": "payments-api",
    "url": "http://localhost:3001",
    "active_milestone": "v12-checkout-flow",
    "feature_count": 3,
    "agent_running": false,
    "last_seen": "2026-03-04T06:00:00Z",
    "status": "online"
  }
]
```

Status values: `"online"` (last seen < 30s), `"stale"` (30–90s), `"offline"` (90s–5min).
Entries older than 5 minutes are removed by the sweep task and never appear in this response.

### 4. Hub Sweep Task

A background task runs every 15 seconds:
- Entries with `last_seen > 90s` are marked `"offline"` (still shown, greyed out).
- Entries with `last_seen > 5min` (300s) are removed from the registry entirely.
- On each sweep, an SSE `ProjectUpdated` or `ProjectRemoved` event is emitted if the registry changed.

### 5. GET /api/hub/events (SSE)

SSE stream for hub UI clients. Emits:
- `ProjectUpdated` — when a heartbeat is received (registration or update).
- `ProjectRemoved` — when an entry is removed by the sweep task.

Event format follows the existing `/api/events` pattern: typed `event:` name + JSON `data:`.

### 6. State Persistence

The hub persists the registry to `~/.sdlc/hub-state.yaml` on each heartbeat and sweep,
so a restart shows cached project cards while pods re-register.

The file is a warm cache, not the source of truth: on startup the hub loads it and marks all
entries as `"offline"` until a heartbeat confirms them.

Persistence format:
```yaml
projects:
  - name: payments-api
    url: "http://localhost:3001"
    active_milestone: v12-checkout-flow
    feature_count: 3
    agent_running: false
    last_seen: "2026-03-04T06:00:00Z"
```

Writes go through `sdlc_core::io::atomic_write` to prevent partial writes.

## Technical Design

### HubState in AppState

Add an optional `hub_registry` field to `AppState`:

```rust
pub hub_registry: Option<Arc<Mutex<HubRegistry>>>
```

`HubRegistry` contains:
- `projects: HashMap<String, ProjectEntry>` — keyed by `url`
- `event_tx: broadcast::Sender<HubSseMessage>` — separate channel for hub SSE

`HubRegistry` is `None` in project mode; `Some(...)` in hub mode.

### New Files

- `crates/sdlc-server/src/hub.rs` — `HubRegistry`, `ProjectEntry`, sweep task, persistence
- `crates/sdlc-server/src/routes/hub.rs` — route handlers for heartbeat, projects, events

### Route Registration

Hub routes are registered only when hub mode is active (via a separate `build_hub_router` function
or conditional route registration in `build_router_from_state`).

### CLI Integration

`sdlc serve --hub` or `sdlc ui start --hub` activates hub mode. A `hub` bool is threaded
through to `serve()` / `serve_on()` and to `AppState`.

## Non-Goals

- No cross-project agents or aggregate dashboards.
- No project management from the hub.
- No Kubernetes API calls.
- No groups, favorites, or sorting beyond latest-beat-first.

## Acceptance Criteria

1. `POST /api/hub/heartbeat` with valid payload returns 200 and registers the project.
2. `GET /api/hub/projects` returns the registered project with correct `status`.
3. After 90 seconds without a heartbeat, the project status becomes `"offline"`.
4. After 300 seconds without a heartbeat, the project is removed from the registry.
5. `GET /api/hub/events` emits `ProjectUpdated` on each heartbeat.
6. Hub state persists to `~/.sdlc/hub-state.yaml` and loads on restart.
7. In hub mode, project-specific routes (`/api/features`, `/api/milestones`, etc.) are not registered.
8. The `--hub` flag on `sdlc ui start` correctly activates hub mode.
