# Blocked Feature UX — Design Decision Record

## Resolved: What the Panel Shows

The feature detail page gets a **BlockedPanel** component that appears when `feature.blocked === true`. It renders at the top of the feature detail view with three zones:

### Zone 1: Blocker list

Each blocker in `feature.blockers: Vec<String>` is listed with:
- The blocker text
- If the text exactly matches a known in-project feature slug → render as a link `→ Go to [slug]`
- A `✕ Remove` affordance with optional inline reason input (appears on click, not a modal)

When Remove is submitted: `DELETE /api/features/:slug/blockers/:idx` with optional `{ reason }` body.
If reason provided: also `POST /api/features/:slug/comments` with `{ flag: 'decision', body: 'Blocker removed: <reason>' }`.

### Zone 2: Run with direction

Primary action. Text input + `▶ Run with direction` button.
Posts to existing `POST /api/run/:slug` with `{ context: '<direction>' }`.
No changes to the agent prompt template — agent reads `sdlc next`, sees blocked state, applies user direction.
After submit: button disables, existing activity feed appears (same as normal Run flow).

### Zone 3: (Implicit)

No 'check now' button. SSE file watcher on `.sdlc/features/<slug>/` detects any mutation and emits `SseMessage::Update`. Frontend re-fetches on Update. Consistent with all other mutation patterns.

---

## Resolved: What We're NOT Building

- No cross-project dependency navigation (Jordan: ignore cross-project)
- No 'check now' button (SSE handles auto-refresh)
- No new SSE variant (generic `Update` is sufficient for blocker mutations)
- No modal for reason (inline, optional)
- No 'waive' terminology for blocker removal (it's Remove, not Waive — artifacts have waive; blockers don't)
- No CLI `sdlc feature unblock` command in v1 (UI story doesn't require it)

---

## Implementation Footprint

| Layer | Change |
|---|---|
| `sdlc-core` | `Feature::remove_blocker(idx: usize) -> SdlcResult<()>` + save |
| `sdlc-server` | `DELETE /api/features/:slug/blockers/:idx` + register in `lib.rs` |
| Frontend | `BlockedPanel.tsx` (~80 lines) |
| Frontend | Feature detail page: import + render when `feature.blocked` |

Total: ~115 lines of new code.