# Tasks: Dashboard Horizon Zone

## T1 — Implement HorizonZone component

Replace the stub `HorizonZone` in
`frontend/src/components/dashboard/HorizonZone.tsx` with the full implementation
from the design.

Deliverables:
- Props: `{ milestones: MilestoneSummary[], featureBySlug: Map<string, FeatureSummary> }`
- Filter `milestones` to horizon set: milestones where every assigned feature is in
  `draft` phase (or milestone has no features)
- Fetch active ponders via `api.getRoadmap()` in a `useEffect`; filter to
  `exploring` or `converging` status; catch errors silently
- Render `null` when both lists are empty
- Render section with "Horizon" label + `Telescope` icon
- Two sub-sections in a single card: "Upcoming Milestones" and "Active Ponders"
- Each milestone row: title link, `StatusBadge`, feature count
- Each ponder row: title link, `StatusBadge`, up to 2 tag chips, copy button
  for `/sdlc-ponder <slug>` command
- Inline `CopyButton` sub-component with 1.5s copied feedback
- All links use `react-router-dom` `<Link>` (no full page navigation)

## T2 — Wire HorizonZone into Dashboard

Update `frontend/src/pages/Dashboard.tsx` to pass props to `<HorizonZone>`:

```tsx
<HorizonZone
  milestones={activeMilestones}
  featureBySlug={featureBySlug}
/>
```

Both values already exist in the Dashboard scope. No new state, effects, or imports
required in `Dashboard.tsx` beyond updating the component call.

## T3 — TypeScript + build verification

Run `cd frontend && npx tsc --noEmit` to verify no type errors. Fix any type errors
found. The build must be clean.
