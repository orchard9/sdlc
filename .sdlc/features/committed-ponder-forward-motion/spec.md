# Spec: Committed Ponder Forward Motion

## Problem

When a ponder is committed (crystallized into milestones via `/sdlc-ponder-commit`), the UI shows a green "committed" badge but provides no forward action. The user hits a dead end — there's no visible link to the created milestones and no way to trigger the next logical action (preparing those milestones) without navigating away manually.

The `PonderDetail.committed_to: string[]` field already contains the milestone slugs, but this data is not surfaced in the UI.

## Solution

Add two UI elements to the ponder detail view for committed ponders:

### 1. Milestone Links Section

When `entry.status === 'committed'` and `entry.committed_to` has entries, render a section showing each committed milestone as a clickable link navigating to `/milestones/{slug}`. This makes the output of the commitment visible and navigable.

### 2. Prepare Action Button

Add a prominent button in the ponder header that triggers `/api/milestone/{slug}/prepare` for the first committed milestone. This follows the existing pattern from `WavePlan.tsx` — uses `useAgentRuns()` with `runType: 'milestone_prepare'`, shows loading state during the agent run.

## Scope

- **In scope:** Milestone links display, prepare button, loading states, navigation
- **Out of scope:** Changes to the ponder commit flow itself, milestone prepare backend (already exists), changes to non-committed ponder states

## Affected Components

| Component | Change |
|---|---|
| `frontend/src/pages/PonderPage.tsx` (`EntryDetailPane`) | Add milestone links section and prepare button |
| `frontend/src/components/ponder/DialoguePanel.tsx` | Update empty state for committed ponders to show milestone links |

## Acceptance Criteria

1. Committed ponders display all milestone slugs from `committed_to` as clickable links
2. Clicking a milestone link navigates to `/milestones/{slug}`
3. A "Prepare" button is visible for committed ponders with at least one milestone
4. Clicking "Prepare" triggers the milestone prepare agent run via the existing API
5. Button shows loading/running state while the agent executes
6. Non-committed ponders are unaffected
