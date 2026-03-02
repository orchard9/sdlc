## Design: Milestones Page Upgrade

### What changes

**MilestonesPage.tsx**
- Unified page: active milestones at top, collapsible 'Archive' section at bottom for released milestones
- Archive section collapsed by default, toggle shows count: 'Archive · N released'
- Fetch PrepareResult via api.getProjectPrepare() to know the active milestone + waves
- MilestoneCard gets Run Wave button when: activeMilestoneSlug === m.slug && waves.length > 0

**MilestoneCard Run Wave button**
- In the card header row (right side)
- Same pattern as WaveSection in WavePlan.tsx: useAgentRuns hook, isRunning/startRun/focusRun
- Key: `milestone-run-wave:${m.slug}`
- Start URL: `/api/milestone/${m.slug}/run-wave`
- Stop URL: `/api/milestone/${m.slug}/run-wave/stop`
- Shows Loader2 + 'Running' if running, Play + 'Run Wave' if not
- Only visible on the one active milestone with waves

**Sidebar.tsx**
- Remove 'Archive' nav item and Archive icon import

**App.tsx**
- Remove /milestones/archive route

### What does NOT change
- No new API endpoints (PrepareResult already exists)
- No changes to MilestoneSummary type
- No backend changes
- MilestoneDetail page unchanged
- BottomTabBar unchanged (roots already just /milestones)