# Spec: Backlog section in Dashboard frontend

## Overview

Add a Backlog section to `frontend/src/pages/Dashboard.tsx` that surfaces the project-level backlog (stored in `.sdlc/backlog.yaml`) to the user. The backlog captures out-of-scope concerns discovered by agents during runs. This section lets the user triage items without leaving the dashboard.

## Background

The `sdlc-core` backlog data layer already exists (`crates/sdlc-core/src/backlog.rs`). Items have:
- `id` (B1, B2, …)
- `title` — one sentence describing the concern
- `kind` — `concern | idea | debt`
- `status` — `open | parked | promoted`
- `description` — optional multi-line detail
- `evidence` — optional file/line reference
- `source_feature` — optional slug of the feature that discovered it
- `park_reason` — required when status is `parked`
- `promoted_to` — feature slug when `promoted`
- `created_at`, `updated_at`

The server currently has **no `/api/backlog` route**. This feature must add that route and then consume it from the frontend.

## Scope

### Backend (sdlc-server)

1. **Add `/api/backlog` route** — GET, returns all items as JSON. Supports optional query params:
   - `?status=open|parked|promoted` — filter by status
   - `?source_feature=<slug>` — filter by source feature

2. **Add `/api/backlog/:id/park` route** — POST, body `{ park_reason: string }`. Parks the item. Returns updated item or error.

3. **Add `/api/backlog/:id/promote` route** — POST, body `{ slug: string; title: string; description?: string }`. Creates a new feature via `Feature::create(...)` then calls `BacklogStore::mark_promoted`. Returns `{ promoted_to: string }`.

4. Register all three routes in `crates/sdlc-server/src/lib.rs` and add a `routes/backlog.rs` module.

### Frontend

5. **Add `BacklogItem` type** to `frontend/src/lib/types.ts`.

6. **Add `api.getBacklog`, `api.parkBacklogItem`, `api.promoteBacklogItem`** to `frontend/src/api/client.ts`.

7. **Add Backlog section to `Dashboard.tsx`** below the escalations section. Section only renders when there are open items (or when `showParked` is true).

### Backlog section UI requirements

- **Section header**: "Backlog" label + open item count badge (e.g. `3 open`)
- **Empty state**: shown when no open items exist:
  > "When an agent finishes a run, it captures concerns that were out of scope for that feature. Items appear here. Promote to create a tracked feature, or Park to set aside. Nothing in the backlog blocks your current work."
- **Per-item card**:
  - `title` text
  - `kind` badge (color-coded: concern=red-tinted, idea=blue-tinted, debt=amber-tinted)
  - `source_feature` link → `/features/<slug>` (if present)
  - `created_at` relative timestamp
  - Staleness indicator: items older than 60 days show a `Clock` icon with "stale" label in muted amber
  - **Promote CTA**: button with tooltip "Creates a draft feature. No agent work begins automatically." On click: triggers promote flow → shows a small inline form (feature title pre-populated from backlog title, editable) → submit calls `api.promoteBacklogItem` → on success shows a toast notification with a link to the new feature slug
  - **Park CTA**: button. On click: shows an inline textarea prompt for `park_reason`. User must enter a non-empty reason before confirm becomes enabled. On confirm: calls `api.parkBacklogItem`. On success: item disappears from open list.
- **Source feature filter**: dropdown above the items list (if 2+ distinct source features are present) — "All sources" default, then per-slug options. Filters the displayed items client-side.
- **Show parked items toggle**: collapsed by default; a "Show parked (N)" toggle reveals parked items below the open list (read-only, no actions).

## Data flow

- Dashboard fetches backlog once on mount alongside its other setup checks (no separate hook needed — plain `useState` + `useEffect` fetch).
- SSE does not need to trigger backlog refetch (backlog is agent-written, not real-time). A simple refetch after park/promote is sufficient.
- After park or promote, re-fetch `/api/backlog?status=open` to refresh the list.

## Error handling

- Park with empty reason: server returns 422; UI shows inline error "Park reason is required."
- Promote with duplicate slug: server returns 409 (feature already exists); UI shows inline error "A feature with that slug already exists. Choose a different name."
- Network errors: show inline error message on the affected card.

## Out of scope

- Editing backlog item title or description from the UI (agents write items)
- Batch operations
- Backlog page (a dedicated route `/backlog`) — this feature is dashboard-only
- Notification/badge in sidebar for backlog count

## Acceptance criteria

1. GET `/api/backlog` returns items from `.sdlc/backlog.yaml` (empty list when file absent, not an error).
2. POST `/api/backlog/:id/park` with a valid reason updates item status to `parked`.
3. POST `/api/backlog/:id/promote` creates a new draft feature and marks the item as `promoted`.
4. Dashboard shows Backlog section with open item count badge.
5. Empty state copy is shown verbatim when no open items exist.
6. Each item card shows title, kind badge, source_feature link (if set), timestamp, and staleness indicator for items > 60 days old.
7. Park CTA requires a non-empty reason before the confirm is enabled.
8. Promote CTA shows tooltip "Creates a draft feature. No agent work begins automatically." and, on success, shows a toast with a link to the new feature.
9. Source feature filter dropdown appears when 2+ distinct source features are present.
10. Parked items are hidden by default; "Show parked (N)" toggle reveals them.
