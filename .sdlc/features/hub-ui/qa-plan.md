# QA Plan: Hub UI

## Scope

Verify that the hub UI page renders correctly in hub mode, updates live via SSE, and that normal mode is unaffected.

## Test Cases

### 1. Hub mode detection

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 1.1 | Hub mode active | Server running with hub registry; open app at `/` | `HubPage` renders, no sidebar visible |
| 1.2 | Normal mode | Server running without hub mode; open app at `/` | Normal `Dashboard` renders with sidebar |
| 1.3 | Detection loading | While detection fetch is in-flight | Full-screen spinner shown, no flash of wrong content |

### 2. Project listing

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 2.1 | Projects present | Hub mode with registered projects | Cards render with name, URL, status dot |
| 2.2 | No projects | Hub mode with empty registry | Empty state with `~/.sdlc/hub.yaml` hint shown |
| 2.3 | Online status | Project with `last_seen < 30s` | Green status dot |
| 2.4 | Stale status | Project with `last_seen 30–90s` | Yellow status dot |
| 2.5 | Offline status | Project with `last_seen > 90s` | Grey status dot |
| 2.6 | Agent badge | Project with `agent_running: true` | Pulsing green badge visible |
| 2.7 | No agent badge | Project with `agent_running: false` or null | No agent badge |
| 2.8 | Milestone badge | Project with `active_milestone` set | Milestone slug badge visible |
| 2.9 | No milestone | Project with `active_milestone: null` | No milestone badge |
| 2.10 | Feature count | Project with `feature_count: 5` | "5 features" text visible |

### 3. Filter

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 3.1 | Filter by name | Type partial name (case insensitive) | Only matching cards shown |
| 3.2 | Filter by URL | Type partial URL | Only matching cards shown |
| 3.3 | Count with filter | Filter with 1 of 4 matches | Count shows "1 of 4 projects" |
| 3.4 | Count without filter | Empty filter input | Count shows "4 projects" |
| 3.5 | Filter no match | Type something with no matches | Empty grid, count "0 of N projects" |
| 3.6 | Clear filter | Clear the filter input | All cards reappear |

### 4. Card navigation

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 4.1 | Click card | Click any project card | `window.open(url, '_blank')` called — new tab opens |

### 5. Live SSE updates

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 5.1 | New project via SSE | `project_updated` event for new URL | New card appears without page reload |
| 5.2 | Update existing | `project_updated` event for known URL | Existing card updates (name, status, etc.) |
| 5.3 | Project removed | `project_removed` event | Card disappears without page reload |
| 5.4 | Status recomputation | Client-side 15s interval fires | Status dots update based on `last_seen` age |

### 6. Normal mode unaffected

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 6.1 | Existing routes work | Normal mode — navigate to `/features` | Features page renders normally |
| 6.2 | No regressions | All existing pages load | No JavaScript errors |

## Build Verification

- `SDLC_NO_NPM=1 cargo test --all` passes (no Rust changes expected)
- `cd frontend && npm run build` succeeds (no TypeScript errors)
- `cargo clippy --all -- -D warnings` passes

## Manual Smoke Test

1. Start sdlc-server without `--hub-mode`; open app — verify normal mode
2. Start sdlc-server with `--hub-mode`; open app — verify HubPage
3. Register a project via heartbeat API — verify card appears
4. Wait > 30s without heartbeat — verify status dot changes to stale
5. Send new heartbeat — verify dot returns to green
