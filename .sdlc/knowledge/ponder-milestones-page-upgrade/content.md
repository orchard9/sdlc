---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Design settled — unified milestones page, archive section, run wave on active milestone"
  next: "Build it — MilestonesPage.tsx refactor, Sidebar.tsx cleanup, App.tsx route removal"
  commit: "Design is complete, no open questions. Ready to implement."
---

## Session 1 — Milestones page upgrade

### Brief

User wants to upgrade the milestones page:
1. Single page showing all milestones — active at top, released below under collapsible Archive section
2. Remove the separate `/milestones/archive` route and "Archive" sidebar nav entry
3. Add "Run Wave" button on each milestone card where applicable (similar to Dashboard wave plan)

### Code audit

Read all relevant files:
- `frontend/src/pages/MilestonesPage.tsx` — `filter` prop, two routes, no wave support
- `frontend/src/components/features/WavePlan.tsx` — Run Wave button pattern, uses `useAgentRuns`, `milestoneSlug`, `isCurrentWave`
- `frontend/src/components/features/PreparePanel.tsx` — fetches `api.getProjectPrepare()`, passes `result.milestone` to WavePlan
- `frontend/src/components/layout/Sidebar.tsx` — has both "Milestones" and "Archive" items
- `frontend/src/App.tsx` — `/milestones` and `/milestones/archive` routes
- `frontend/src/lib/types.ts` — `MilestoneSummary` (no wave data), `PrepareResult` (has milestone + waves)

### Design decisions

⚑ Decided: Single `/milestones` route. Active milestones at top. Released milestones in collapsible "Archive" section (default collapsed, toggle shows count). This matches the Dashboard's own Archive treatment.

⚑ Decided: Run Wave button uses PrepareResult cross-reference. `prepareResult.milestone === m.slug && prepareResult.waves.length > 0` determines if the button shows. No new API, no changes to MilestoneSummary.

⚑ Decided: Three files only — MilestonesPage.tsx, Sidebar.tsx, App.tsx. No backend changes.

### Thought partner perspectives

**Ben Hartley · UX/Productivity:** Single-page with collapsible archive is the right UX. Two nav items for milestones is splitting attention unnecessarily.

**Tobias Krenn · Skeptic:** Don't add per-milestone wave API endpoints. PrepareResult is sufficient.

**Dan Reeves · Minimalist:** Three files. No new API. No new types. Reuse existing hooks.

### Implementation plan

**MilestonesPage.tsx:**
- Remove `filter` prop
- Fetch `PrepareResult` via `api.getProjectPrepare()` (use `useCallback` + `useEffect` + `useSSE` pattern)
- Split milestones: active = status !== 'released', released = status === 'released'
- Render active milestones first
- Render collapsible Archive section with `useState(false)` for expand/collapse
- MilestoneCard gets `isActiveMilestone` + `isRunWaveAvailable` props for the Run Wave button

**MilestoneCard Run Wave button:**
- In header row, right side
- Uses `useAgentRuns()` — `isRunning`, `startRun`, `focusRun`, `getRunForKey`
- Key: `milestone-run-wave:${m.slug}`
- Start URL: `/api/milestone/${m.slug}/run-wave`
- Stop URL: `/api/milestone/${m.slug}/run-wave/stop`
- Renders: Loader2 + "Running" if running, Play + "Run Wave" if not

**Sidebar.tsx:**
- Remove Archive nav item
- Remove Archive icon from lucide import

**App.tsx:**
- Remove `/milestones/archive` route

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Design settled — unified milestones page, archive section, run wave on active milestone"
  next: "Build it — MilestonesPage.tsx refactor, Sidebar.tsx cleanup, App.tsx route removal"
  commit: "Design is complete, no open questions. Ready to implement."
---

## Session 1 — Milestones page upgrade

### Brief

User wants to upgrade the milestones page:
1. Single page showing all milestones — active at top, released below under collapsible Archive section
2. Remove the separate `/milestones/archive` route and "Archive" sidebar nav entry
3. Add "Run Wave" button on each milestone card where applicable (similar to Dashboard wave plan)

### Code audit

Read all relevant files:
- `frontend/src/pages/MilestonesPage.tsx` — `filter` prop, two routes, no wave support
- `frontend/src/components/features/WavePlan.tsx` — Run Wave button pattern, uses `useAgentRuns`, `milestoneSlug`, `isCurrentWave`
- `frontend/src/components/features/PreparePanel.tsx` — fetches `api.getProjectPrepare()`, passes `result.milestone` to WavePlan
- `frontend/src/components/layout/Sidebar.tsx` — has both "Milestones" and "Archive" items
- `frontend/src/App.tsx` — `/milestones` and `/milestones/archive` routes
- `frontend/src/lib/types.ts` — `MilestoneSummary` (no wave data), `PrepareResult` (has milestone + waves)

### Design decisions

⚑ Decided: Single `/milestones` route. Active milestones at top. Released milestones in collapsible "Archive" section (default collapsed, toggle shows count). This matches the Dashboard's own Archive treatment.

⚑ Decided: Run Wave button uses PrepareResult cross-reference. `prepareResult.milestone === m.slug && prepareResult.waves.length > 0` determines if the button shows. No new API, no changes to MilestoneSummary.

⚑ Decided: Three files only — MilestonesPage.tsx, Sidebar.tsx, App.tsx. No backend changes.

### Thought partner perspectives

**Ben Hartley · UX/Productivity:** Single-page with collapsible archive is the right UX. Two nav items for milestones is splitting attention unnecessarily.

**Tobias Krenn · Skeptic:** Don't add per-milestone wave API endpoints. PrepareResult is sufficient.

**Dan Reeves · Minimalist:** Three files. No new API. No new types. Reuse existing hooks.

### Implementation plan

**MilestonesPage.tsx:**
- Remove `filter` prop
- Fetch `PrepareResult` via `api.getProjectPrepare()` (use `useCallback` + `useEffect` + `useSSE` pattern)
- Split milestones: active = status !== 'released', released = status === 'released'
- Render active milestones first
- Render collapsible Archive section with `useState(false)` for expand/collapse
- MilestoneCard gets `isActiveMilestone` + `isRunWaveAvailable` props for the Run Wave button

**MilestoneCard Run Wave button:**
- In header row, right side
- Uses `useAgentRuns()` — `isRunning`, `startRun`, `focusRun`, `getRunForKey`
- Key: `milestone-run-wave:${m.slug}`
- Start URL: `/api/milestone/${m.slug}/run-wave`
- Stop URL: `/api/milestone/${m.slug}/run-wave/stop`
- Renders: Loader2 + "Running" if running, Play + "Run Wave" if not

**Sidebar.tsx:**
- Remove Archive nav item
- Remove Archive icon from lucide import

**App.tsx:**
- Remove `/milestones/archive` route
