# Tasks: milestones-unified-page

## Implementation

- [ ] Refactor `MilestonesPage.tsx`: remove `filter` prop; split milestones into active and released arrays; render active list first; add collapsible archive section (default collapsed, toggle with count); fetch `PrepareResult` via `api.getProjectPrepare()` to determine active milestone and wave availability
- [ ] Add Run Wave button to `MilestoneCard`: show only when `activeMilestoneSlug === m.slug && waves.length > 0`; use `useAgentRuns()` hook; key `milestone-run-wave:${m.slug}`; start URL `/api/milestone/${m.slug}/run-wave`; stop URL `/api/milestone/${m.slug}/run-wave/stop`; show Loader2+"Running" if running, Play+"Run Wave" if not
- [ ] Remove Archive nav item from `Sidebar.tsx` and clean up unused `Archive` icon import
- [ ] Remove `/milestones/archive` route from `App.tsx`

## Verification

- [ ] Navigate to `/milestones` — active milestones appear at top, archive section at bottom collapsed
- [ ] Expand archive — released milestones render with same card style
- [ ] Archive count in toggle matches number of released milestones
- [ ] Sidebar shows only "Milestones" (no "Archive" entry)
- [ ] `/milestones/archive` returns 404 or redirects (route removed)
- [ ] Run Wave button visible on active milestone card when a wave plan exists
- [ ] Run Wave button not visible on released milestone cards
