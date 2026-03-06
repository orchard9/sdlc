# Spec: fleet-management-ui

## Summary

Evolve the existing `HubPage.tsx` from a read-only heartbeat-based project list into a full fleet control plane UI. The page becomes the single entry point at `sdlc.threesix.ai` where authenticated users can see running instances, discover available repos, import external repos, and monitor fleet-wide agent activity — all in real time via SSE.

## Context

The existing `HubPage` displays heartbeat-reporting projects with status dots, filtering, and SSE updates. The fleet-management-api feature (sibling in this milestone) adds three new endpoints that this UI must consume:

| Endpoint | Purpose |
|---|---|
| `GET /api/hub/fleet` | Running instances with k8s status, pod health, active milestone, agent info |
| `GET /api/hub/available` | Gitea org repos without running instances |
| `POST /api/hub/provision` | Start an instance for an available repo |
| `POST /api/hub/import` | Import external repo URL, mirror to Gitea, auto-provision |
| `GET /api/hub/agents` | Fleet-wide aggregate agent run counts |

## Requirements

### R1: Three-section dashboard layout
The page is organized into three visual sections:
1. **Running Instances** — cards for projects with live k8s deployments
2. **Available Repos** — cards for Gitea org repos that have no instance yet
3. **Import** — form to bring in an external git repo

### R2: Running instances section
- Each card shows: project name, URL (clickable, opens in new tab), health status dot (green=healthy, yellow=degraded, red=failing, grey=unknown), active milestone label, feature count, agent-running badge with pulse animation
- Data source: `GET /api/hub/fleet`
- Cards are clickable — navigate to the instance URL
- SSE `FleetUpdated` events update cards in place without full reload

### R3: Available repos section
- Each card shows: repo name, description (truncated), "Start" button
- Data source: `GET /api/hub/available`
- "Start" button calls `POST /api/hub/provision` with the repo slug
- After clicking Start, the card shows a "Provisioning..." state (spinner + disabled button)
- On SSE update confirming the instance is running, the repo moves from Available to Running
- First-time user context: a one-liner above the section explains what starting an instance does (e.g., "Start deploys an sdlc workspace for this repo in the fleet")

### R4: Import section
- Simple form: URL text input + optional PAT input + Import button
- Calls `POST /api/hub/import` with `{ url, pat? }`
- Shows progress states: importing → provisioning → done
- On success, the new instance appears in the Running section via SSE
- Error state: inline error message below the form

### R5: Search-first design
- Search input is at the top of the page, autofocused on mount
- Filters across both Running and Available sections simultaneously
- Client-side text filter on name, URL, and description fields
- Instant filtering (no debounce needed at expected scale of <200 repos)

### R6: Fleet-wide agent activity summary
- Header area shows aggregate agent run count: "N agents running across M projects"
- Data source: `GET /api/hub/agents` on initial load, updated via SSE
- If zero agents are running, the line reads "No active agents"

### R7: SSE integration
- Reuse/extend `useHubSSE` hook to handle new event types from fleet endpoints
- Events: `FleetUpdated` (instance status change), `FleetProvisioned` (new instance), `FleetAgentStatus` (agent count change)
- No polling — all live updates come through SSE

### R8: Empty states
- Running section empty: "No instances running. Start one from available repos below."
- Available section empty: "All repos have running instances."
- Both empty: current empty state with setup instructions

## Out of scope
- Instance deletion or teardown from UI
- Per-user access control or role-based visibility
- Instance configuration or settings editing
- Log viewing or debugging from the hub
- Mobile-optimized layout (desktop-first for v1)

## Dependencies
- `fleet-management-api` — all data endpoints this UI consumes
- `fleet-auth-gate` — authentication context (user is already authenticated when they reach this page)
