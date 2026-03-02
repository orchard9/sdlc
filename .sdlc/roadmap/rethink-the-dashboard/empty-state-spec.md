# Empty State — Design Spec

## Two distinct states

### State A: Orchestrator idle, Horizon not empty
- Trigger: no features actively running, but queued milestones exist in Horizon
- Shows: amber status banner at top of page
  - Text: "Orchestrator idle — N milestones ready"
  - Button: [Run wave] (conditional — only if orchestrator has trigger API; TBD)
- This is NOT the empty state component — the rest of the dashboard renders normally

### State B: True empty — nothing in flight AND Horizon is empty
- Trigger: no running features, no queued milestones, no active/converging ponders
- Replaces the current DashboardEmptyState component

## State B layout (three blocks)

```
+------------------------------------------------------+
|  The project is caught up.                          |
|  N milestones shipped. Nothing queued.              |
+------------------------------------------------------+
|  What to think about next:                         |
|  [Suggestion chip 1]  from /sdlc-suggest            |
|  [Suggestion chip 2]  from /sdlc-suggest            |
|  [Suggestion chip 3]  from /sdlc-suggest            |
|                                                     |
|  [+ Start a new ponder]                            |
+------------------------------------------------------+
|  Or look back:                                     |
|  · View shipped milestones                        |
|  · Review filed tasks without a milestone         |
+------------------------------------------------------+
```

## Suggestion content source
- Dashboard calls `/api/suggest` endpoint when it detects State B
- Renders top 3 results as clickable chips
- Each chip routes to `/sdlc-ponder` or `/sdlc-plan` depending on suggestion type
- This is a data fetch against existing `/sdlc-suggest` output — not a new AI inference call

## Emotional design principle
The empty state should feel like a **clean desk**, not a 404. The accomplishment signal ("project is caught up") comes first, before any "what to do next" content.