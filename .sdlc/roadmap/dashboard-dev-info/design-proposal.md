# Design Proposal: Dashboard Dev Info

## Tier 1 — Running Indicator (low effort, high value)

Add a pulsing indicator to FeatureCard and MilestoneDigestRow when an agent is actively running.

**Feature-level:** Use existing `isRunning(feature.slug)` from AgentRunContext. Show a small animated dot or spinner next to the phase badge.

**Milestone-level:** Aggregate — a milestone is 'running' if ANY of its features has an active run, OR if a UAT run is active. Show count: '2 agents running'.

**Data needed:** Already available. Zero backend changes.

## Tier 2 — Run Details on Hover/Click (medium effort)

On hover or click of the running indicator, show:
- Run type (feature run, UAT, ponder)
- Started at (relative time: '3m ago')
- Turn count (if available from SSE updates)
- Cost so far

**Data needed:** `getRunForKey()` already returns full RunRecord.

## Tier 3 — User Attribution (requires backend change)

Add `created_by: Option<String>` to RunRecord. Populate from:
- Auth context (cluster mode: OAuth user from session)
- `git config user.name` (local mode: fallback)
- Display as avatar or initials next to the running indicator

**Breaking change consideration:** RunRecord is serialized to JSON files. Adding an optional field is backward-compatible (old records just won't have it).

## Rejected Alternatives

- **Polling /api/runs:** Wasteful when SSE already pushes updates
- **Separate 'who's working' API:** Over-engineering — just add the field to RunRecord
- **Global activity feed on dashboard:** Out of scope — this is about per-card indicators, not a feed