# Design: fleet-management-ui

## Architecture

This feature evolves the existing `HubPage.tsx` in-place. In hub mode, `App.tsx` renders `<HubPage />` as the entire application (no router, no AppShell). This stays the same — the hub page remains a standalone full-page component.

### Component hierarchy

```
HubPage (pages/HubPage.tsx) — top-level, owns all state
├── AgentSummaryBar — "N agents running across M projects"
├── SearchInput — autofocused filter input
├── RunningSection — grid of FleetInstanceCard
│   └── FleetInstanceCard — status dot, name, URL, milestone, agent badge
├── AvailableSection — grid of AvailableRepoCard
│   └── AvailableRepoCard — repo name, description, Start button
├── ImportSection — URL+PAT form, Import button, progress states
└── EmptyState — shown when both sections are empty
```

### File changes

| File | Change |
|---|---|
| `frontend/src/pages/HubPage.tsx` | Rewrite: three sections, agent summary, import form |
| `frontend/src/lib/types.ts` | Add `FleetInstance`, `AvailableRepo`, `FleetAgentSummary`, extend `HubSseEvent` |
| `frontend/src/api/client.ts` | Add `getFleet()`, `getAvailable()`, `getAgentSummary()`, `provision()`, `importRepo()` |
| `frontend/src/hooks/useHubSSE.ts` | Extend to handle `fleet_updated`, `fleet_provisioned`, `fleet_agent_status` events |

No new pages or routes — this is all within the existing `HubPage` rendered in hub mode.

### Data types

```typescript
export interface FleetInstance {
  name: string
  url: string
  namespace: string
  status: 'healthy' | 'degraded' | 'failing' | 'unknown'
  pod_status: string          // "Running", "CrashLoopBackOff", etc.
  active_milestone: string | null
  feature_count: number | null
  agent_running: boolean
  active_agent_runs: number
  created_at: string          // ISO-8601
}

export interface AvailableRepo {
  name: string
  slug: string
  description: string | null
  url: string                 // Gitea repo URL
}

export interface FleetAgentSummary {
  total_active_runs: number
  projects_with_agents: number
}

// Extended SSE event types
export interface HubSseEvent {
  type: 'project_updated' | 'project_removed' | 'fleet_updated' | 'fleet_provisioned' | 'fleet_agent_status'
  project?: HubProjectEntry
  instance?: FleetInstance
  url?: string
  agent_summary?: FleetAgentSummary
}
```

### API client additions

```typescript
// In api/client.ts
getFleet: () => request<FleetInstance[]>('/api/hub/fleet'),
getAvailable: () => request<AvailableRepo[]>('/api/hub/available'),
getAgentSummary: () => request<FleetAgentSummary>('/api/hub/agents'),
provision: (slug: string) => request<{ status: string }>('/api/hub/provision', {
  method: 'POST', body: JSON.stringify({ slug }),
  headers: { 'Content-Type': 'application/json' },
}),
importRepo: (url: string, pat?: string) => request<{ status: string }>('/api/hub/import', {
  method: 'POST', body: JSON.stringify({ url, pat }),
  headers: { 'Content-Type': 'application/json' },
}),
```

### State management

All state is local to `HubPage` via `useState`:
- `instances: FleetInstance[]` — from `GET /api/hub/fleet`
- `available: AvailableRepo[]` — from `GET /api/hub/available`
- `agentSummary: FleetAgentSummary` — from `GET /api/hub/agents`
- `filter: string` — search text
- `provisioningSlug: string | null` — repo currently being provisioned
- `importState: 'idle' | 'importing' | 'provisioning' | 'done' | 'error'`
- `importError: string | null`

### SSE flow

The existing `useHubSSE` hook is extended:
1. `fleet_updated` — upsert instance in `instances` array by namespace
2. `fleet_provisioned` — add to `instances`, remove from `available`, clear `provisioningSlug`
3. `fleet_agent_status` — update `agentSummary`
4. Existing `project_updated` / `project_removed` are kept as fallback for backward compat

### Layout

```
┌─────────────────────────────────────────────────┐
│  [🔍 Search projects...                       ] │  ← autofocused
│  3 agents running across 2 projects             │  ← AgentSummaryBar
├─────────────────────────────────────────────────┤
│  Running (12)                                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ 🟢 sdlc  │ │ 🟢 myapp │ │ 🟡 foo  │        │
│  │ v42-fl.. │ │ v3-init  │ │ v1-mvp   │        │
│  │ 5 feats  │ │ 2 feats  │ │ agent 🟢 │        │
│  └──────────┘ └──────────┘ └──────────┘        │
├─────────────────────────────────────────────────┤
│  Available (8)                                  │
│  Start deploys an sdlc workspace for this repo. │  ← first-time context
│  ┌──────────────────┐ ┌──────────────────┐      │
│  │ my-lib            │ │ data-pipeline     │      │
│  │ A utility library │ │ ETL framework     │      │
│  │ [Start]           │ │ [Start]           │      │
│  └──────────────────┘ └──────────────────┘      │
├─────────────────────────────────────────────────┤
│  Import External Repo                           │
│  ┌──────────────────────────────────────┐       │
│  │ https://github.com/org/repo          │       │
│  └──────────────────────────────────────┘       │
│  ┌──────────────────────────────────────┐       │
│  │ PAT (optional, for private repos)    │       │
│  └──────────────────────────────────────┘       │
│  [Import]                                       │
└─────────────────────────────────────────────────┘
```

### Graceful degradation

- If `GET /api/hub/fleet` fails (API not yet deployed), fall back to existing heartbeat data from `GET /api/hub/projects`
- If `GET /api/hub/available` fails, hide the Available section
- If `GET /api/hub/agents` fails, hide the agent summary bar
- Each section loads independently — one failure does not block the others

### Styling

Follows existing codebase conventions:
- Tailwind utility classes, dark theme (`bg-background`, `text-foreground`, `border-border`)
- Card pattern: `bg-card border border-border rounded-xl p-4 hover:border-zinc-600`
- Grid: `grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4`
- Inputs: `bg-muted/50 border border-border rounded-lg px-3 py-2 text-sm`
- Status dots: colored circles with `rounded-full` (reuse existing `StatusDot`)
