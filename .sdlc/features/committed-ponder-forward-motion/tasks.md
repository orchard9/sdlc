# Tasks: Committed Ponder Forward Motion

## T1: Add milestone links banner to PonderPage EntryDetailPane

Insert a committed-milestones banner between the header and the desktop layout in `PonderPage.tsx`. Renders when `entry.status === 'committed' && entry.committed_to.length > 0`. Each milestone slug is a `<Link>` to `/milestones/{slug}`. Add `Link` import from react-router-dom.

## T2: Add Prepare button to PonderPage header

Add a Prepare button in the header area (next to `StatusBadge`) for committed ponders. Wire `useAgentRuns()` with key `milestone-prepare:{committed_to[0]}` following the WavePlan pattern. Add `Play` import from lucide-react.

## T3: Update DialoguePanel empty state for committed ponders

In `DialoguePanel.tsx`, when `entry.status === 'committed'` and `committed_to` has entries, show milestone links in the empty state instead of hiding all actions. Add `Link` import from react-router-dom.

## T4: Verify build and type-check

Run `cd frontend && npx tsc --noEmit` and fix any type errors introduced.
