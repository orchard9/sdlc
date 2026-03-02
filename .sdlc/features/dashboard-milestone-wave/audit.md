# Audit: Active Milestones and Run Wave on Dashboard

## Scope

Single file changed: `frontend/src/pages/Dashboard.tsx`

- Added import: `MilestonePreparePanel`
- Removed: `CommandBlock` import, `useAgentRuns` import, `isRunning` destructuring
- Replaced: `isComplete`/`nextFeature`/`cmd`/`CommandBlock` block with `<MilestonePreparePanel milestoneSlug={milestone.slug} />`

## Security Surface

This change is purely presentational — it swaps one React component for another in the Dashboard render tree. No new data is fetched, no new API endpoints are called, and no user input is handled in the changed code.

`MilestonePreparePanel` was already in production use on the Milestones page (`MilestonesPage.tsx`) and has been through prior review. No new attack surface is introduced by reusing it here.

## Findings

### Authentication & Authorization
No change. The Dashboard is already behind the same auth as the Milestones page. `MilestonePreparePanel` calls `GET /api/prepare?milestone=<slug>` — the same endpoint already called by `MilestonesPage`. No privilege escalation possible.

### Input Handling
No user input handled in the changed code. `milestoneSlug` is derived from `state.milestones` (server-provided data already used elsewhere on the page).

### XSS
No dangerouslySetInnerHTML or unescaped content. All values rendered through React's JSX escaping. No new rendering paths introduced.

### Data Exposure
No new data fetched. `MilestonePreparePanel` fetches the same prepare endpoint already exposed on the Milestones page. No sensitive data newly surfaced.

### Dependency Risk
No new npm dependencies. Only reuses existing project components.

### Dead Code Removed
`CommandBlock`, `useAgentRuns`, and `isRunning` are removed. This is a net reduction in surface area.

## Verdict

No security findings. Change is safe to merge.
