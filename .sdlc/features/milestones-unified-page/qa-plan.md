# QA Plan: milestones-unified-page

## Scope

Visual + functional verification of the unified milestones page. All checks run against a live `sdlc-server` with an existing `.sdlc/` project that has both active and released milestones.

## Checks

### Navigation
- [ ] Sidebar shows "Milestones" entry — "Archive" entry is gone
- [ ] `/milestones/archive` URL no longer renders a page (no route match)

### Milestones page — active section
- [ ] Active milestones (status ≠ released) appear at the top of `/milestones`
- [ ] Each active milestone card: title link, status badge, vision text, feature chips

### Milestones page — archive section
- [ ] Archive section toggle visible at bottom of page
- [ ] Toggle label includes count of released milestones
- [ ] Archive section is collapsed by default
- [ ] Clicking toggle expands; released milestone cards appear
- [ ] Clicking again collapses

### Run Wave button
- [ ] For a project with an active milestone and a wave plan: Run Wave button appears on the matching card
- [ ] Run Wave button does NOT appear on released (archive) milestone cards
- [ ] Run Wave button does NOT appear on active milestones that are not the current wave plan milestone
- [ ] Clicking Run Wave triggers the run (spinner + "Running" state)
- [ ] Clicking "Running" focuses the run panel

### No regressions
- [ ] MilestoneDetail page still reachable via card title link
- [ ] Feature chip links still navigate to `/features/:slug`
