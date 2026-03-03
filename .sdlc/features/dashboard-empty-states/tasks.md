# Tasks: dashboard-empty-states

## T1: Rewrite DashboardEmptyState with suggestion chips

Update `frontend/src/components/dashboard/DashboardEmptyState.tsx` to accept `hasVision: boolean` and `hasArch: boolean` props and render priority-ordered suggestion chips:
- "Define Vision" chip (links to /setup) — shown when !hasVision
- "Define Architecture" chip (links to /setup) — shown when !hasArch
- "Start a Ponder" chip (links to /ponder?new=1) — shown when hasVision && hasArch
- "Create a Feature directly" chip (links to /features?new=1) — always shown

Each chip is a bordered card with icon, bold label, one-line description, and hover bg-accent effect.

## T2: Update Dashboard.tsx to track hasVision and hasArch separately

In `frontend/src/pages/Dashboard.tsx`:
- Add `hasVision` and `hasArch` state variables (boolean, default false)
- In the existing `useEffect` for vision/arch fetch, set both individually
- Pass `hasVision` and `hasArch` as props to `DashboardEmptyState`

## T3: Add CurrentZone empty state

In `frontend/src/components/dashboard/CurrentZone.tsx`:
- Add a `CurrentZoneEmpty` inline component that renders a soft card prompt when there are no milestones and no ungrouped features
- Message: "No active work. Start a milestone or add a feature."
- Two link buttons: "Milestones" → `/milestones` and "+ Feature" → `/features?new=1`
- Render it when `milestones.length === 0 && ungrouped.length === 0`
