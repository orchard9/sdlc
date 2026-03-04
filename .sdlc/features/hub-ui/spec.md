# Spec: Hub UI — Live Project Listing

## Summary

A React page served at `/` when the server is running in hub mode. Displays registered projects as live-updating cards with filter input, status indicators, and navigation to individual project URLs.

## Context

Hub mode is already implemented server-side (`hub-server-mode` released). The React app needs to detect hub mode and render a dedicated page instead of the normal Dashboard. This page is NOT embedded in the existing sidebar nav — it is a standalone full-page experience.

## API Contracts

### `GET /api/hub/projects`
Returns `ProjectEntry[]` sorted by `last_seen` descending:
```ts
interface ProjectEntry {
  name: string
  url: string
  active_milestone: string | null
  feature_count: number | null
  agent_running: boolean | null
  last_seen: string  // ISO-8601
  status: 'online' | 'stale' | 'offline'
}
```
Returns 503 JSON error `{ error: "not running in hub mode" }` if not in hub mode.

### `GET /api/hub/events`
SSE stream. Event type `hub`. Two event shapes:
```json
{ "type": "project_updated", "project": <ProjectEntry> }
{ "type": "project_removed", "url": "<url>" }
```
Returns 503 if not in hub mode.

### Hub mode detection
`GET /api/hub/projects` returning 200 means hub mode is active. A 503 means normal mode. The app checks this on mount.

## Hub Mode Detection

On app load, the frontend makes a single `GET /api/hub/projects` call:
- If 200: hub mode active — render `HubPage` at `/`
- If 503: normal mode — render normal `Dashboard` at `/`

The `App.tsx` conditionally switches the root route using a `useHubMode()` hook that fetches and caches the hub mode state.

## Page Structure

```
HubPage
├── Header: "Projects" title + N project count badge
├── Filter: text input (case-insensitive match on name/URL)
├── Project count line: "N projects" or "N of M projects"
├── Grid: ProjectCard[]
└── EmptyState (when no projects registered)
```

## ProjectCard

Each card shows:
- **Status dot**: green (online), yellow (stale), grey (offline)
- **Name**: project name (bold)
- **URL**: project URL (small, muted)
- **Active milestone**: slug in a badge (hidden if none)
- **Feature count**: "N features" (hidden if null)
- **Agent badge**: pulsing green dot + "agent running" text (hidden if `agent_running !== true`)
- **Chevron**: right-arrow indicating the card is clickable

Clicking a card navigates to the project URL via `window.open(url, '_blank')`.

## Filter Behaviour

- Text input, client-side filtering
- Case-insensitive match against `name` and `url`
- When filter is active: show "N of M projects"
- When filter is empty: show "N projects"

## Empty State

When the projects list is empty (no entries registered):
- Show icon + "No projects registered"
- Show hint: `Configure projects to send heartbeats. See ~/.sdlc/hub.yaml`

## Live Updates via SSE

Subscribe to `GET /api/hub/events` using a dedicated `useHubSSE` hook (separate from the existing `/api/events` subscriber since hub SSE has a different endpoint and event schema).

On `project_updated`: upsert the project in local state by URL.
On `project_removed`: remove the project from local state by URL.

The hook also runs a local 15-second interval to recompute status client-side based on `last_seen`, so status dots update smoothly without waiting for a server sweep event.

## Routing

Hub mode uses a standalone page with no AppShell/sidebar. When hub mode is detected, the entire app renders `HubPage` at `/` without the normal layout.

## Types

Add to `lib/types.ts`:
```ts
export type HubProjectStatus = 'online' | 'stale' | 'offline'

export interface HubProjectEntry {
  name: string
  url: string
  active_milestone: string | null
  feature_count: number | null
  agent_running: boolean | null
  last_seen: string
  status: HubProjectStatus
}

export interface HubSseEvent {
  type: 'project_updated' | 'project_removed'
  project?: HubProjectEntry
  url?: string
}
```

## Acceptance Criteria

1. Hub mode detection: app shows `HubPage` when `/api/hub/projects` returns 200
2. Normal mode: app shows `Dashboard` when `/api/hub/projects` returns 503
3. Filter input filters cards client-side, case-insensitive
4. Project count text reflects filter state
5. Status dot color matches project status (green/yellow/grey)
6. Agent badge is visible only when `agent_running === true`
7. Clicking a card opens the project URL in a new tab
8. SSE events (project_updated, project_removed) update cards without page reload
9. Empty state shows hint when no projects registered
10. No sidebar/navigation rendered in hub mode
