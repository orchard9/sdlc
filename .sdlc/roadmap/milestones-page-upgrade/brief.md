Upgrade the milestones page:

1. Single milestones page shows ALL milestones — active ones at top, completed (released) ones below under a collapsible 'Archive' section. Remove the separate /milestones/archive route and the Archive sidebar nav entry.

2. For each milestone, show a 'Run Wave' button if applicable based on state — similar to the Dashboard's wave plan treatment.

The current code has: MilestonesPage.tsx with a filter prop, /milestones and /milestones/archive routes, Sidebar has both entries. WavePlan.tsx has the Run Wave button logic. PrepareResult (from /api/prepare) drives which milestone is active and what waves exist.