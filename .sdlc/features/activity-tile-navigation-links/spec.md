# Spec: Activity Tile Navigation Links

## Problem

Activity tiles in the agent panel (`AgentPanel.tsx` → `RunCard.tsx` → `RunActivityFeed.tsx`) are display-only. Each run already tracks its target entity via `RunRecord.run_type` and `RunRecord.target`, but individual tiles provide no way to navigate to the feature, milestone, ponder, or investigation that the run operates against. Users must manually navigate to find the entity a run is working on.

## Solution

Add a clickable navigation link to each run's tile header (the `RunCard` level) that routes the user to the target entity's detail page. The link text shows the entity type and target slug; clicking it navigates via React Router.

## Behavior

1. **RunCard header** gains a navigation chip/link next to the run label displaying the entity type icon and target slug (e.g., "feature/activity-tile-navigation-links").
2. **Routing map** — `run_type` maps to a route prefix:
   - `feature` → `/features/{target}`
   - `milestone_uat` | `milestone_prepare` | `milestone_run_wave` → `/milestones/{target}`
   - `ponder` → `/ponder/{target}`
   - `investigation` → `/investigations/{target}`
   - `vision_align` | `architecture_align` → no link (project-level, not entity-specific)
3. **Click behavior** — standard React Router `<Link>` navigation; no full page reload.
4. **No link when inapplicable** — if `run_type` has no entity page (vision_align, architecture_align) or `target` is empty, no link is rendered.

## Scope

- Frontend only — no backend changes required.
- Modify `RunCard.tsx` to accept and render the navigation link.
- Add a utility function `runTargetRoute(run_type, target) → string | null` for the routing map.
- No changes to individual tile components (`ToolCallCard`, `RunResultCard`, etc.) — the link lives at the run level, not the event level.

## Out of Scope

- Deep-linking to specific artifacts within a feature page.
- Breadcrumb trails or back-navigation from entity pages to the originating run.
- Changes to the `RunRecord` API shape.
