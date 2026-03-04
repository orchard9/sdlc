# Design: Hub UI

## Architecture

Hub mode replaces the normal app shell with a standalone full-screen page. This requires a two-step detection flow in `App.tsx`.

### Hub Mode Detection

A new `useHubMode()` hook fetches `GET /api/hub/projects` once on mount:
- Returns `loading` until the check completes
- Returns `hub` if response is 200 (hub mode active)
- Returns `normal` if response is 503 or any error (fallback to normal mode)

`App.tsx` uses this hook to conditionally render either:
1. `HubPage` (standalone, no AppShell) — when hub mode detected
2. Normal app routes wrapped in `AppShell` — when normal mode

```
App.tsx
  useHubMode() → { mode: 'loading' | 'hub' | 'normal' }
  if loading → full-screen spinner
  if hub → <HubPage />          (no BrowserRouter routes, no sidebar)
  if normal → <AppShell>...</AppShell>  (existing layout unchanged)
```

### File Structure

New files:
- `frontend/src/pages/HubPage.tsx` — main hub page component
- `frontend/src/hooks/useHubSSE.ts` — dedicated SSE hook for `/api/hub/events`

Modified files:
- `frontend/src/App.tsx` — add hub mode detection + conditional routing
- `frontend/src/api/client.ts` — add `getHubProjects()` method
- `frontend/src/lib/types.ts` — add `HubProjectEntry`, `HubProjectStatus`, `HubSseEvent` types

### Data Flow

```
HubPage mounts
  ↓
fetch /api/hub/projects → initial project list
  ↓
useHubSSE subscribes to /api/hub/events
  ↓
On project_updated → upsert by URL in state
On project_removed → delete by URL from state
  ↓
15s interval recomputes status from last_seen
  ↓
Filter input → client-side filter on name + URL
  ↓
Render ProjectCard[] (or EmptyState)
```

### `useHubSSE` Hook

Separate from the existing `useSSE` / `SseContext` infrastructure because:
- Different endpoint (`/api/hub/events` not `/api/events`)
- Different event schema (hub-specific payloads)
- Only relevant in hub mode (conditionally mounted)

The hook manages its own EventSource/fetch connection lifecycle using the same reconnect-on-close pattern as `SseContext`.

### Status Recomputation

The server recomputes status on heartbeat and sweep (every 15s). The frontend mirrors this by running a `setInterval` every 15s that applies the same age rules:
- `< 30s` → `online`
- `30–90s` → `stale`
- `≥ 90s` → `offline`

This provides smooth status transitions without waiting for the next server-emitted event.

## Component Tree

```
HubPage
├── <header> — "Projects" h1 + count badge
├── <input> — filter text input
├── <p> — count line ("N projects" / "N of M projects")
├── <div className="grid"> — ProjectCard grid
│   └── ProjectCard[] — one per visible project
└── EmptyState — shown when projects array is empty
```

### ProjectCard Layout

```
┌─────────────────────────────────────────────┐
│ ●  Project Name                    [chevron]│
│    https://project.example.com             │
│    [milestone-slug] 3 features  ● agent     │
└─────────────────────────────────────────────┘
```

Status dot colors:
- `online` → `bg-green-500`
- `stale` → `bg-yellow-400`
- `offline` → `bg-zinc-500`

Agent badge: pulsing `animate-pulse` green dot + text, only when `agent_running === true`.

## UI States

See [Mockup](mockup.html) for visual reference.

**States modeled:**
1. `hub-projects` — normal state with multiple registered projects
2. `hub-filter` — filter active, showing subset
3. `hub-empty` — no projects registered (empty state with hint)

## Tailwind Classes Reference

Consistent with existing dark-theme pages:
- Background: `bg-background text-foreground`
- Card: `bg-card border border-border rounded-xl`
- Muted text: `text-muted-foreground`
- Filter input: same pattern as `SettingsPage` search inputs
- Grid: `grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4`
