# Spec: ReleasedPanel Component

## Overview

Build a `ReleasedPanel` React component that renders a victory state when a milestone has status `released`. This replaces the generic `VerifyingMini` panel that currently shows for all completed milestones regardless of whether they are still verifying or already released.

## Problem

The `MilestonePreparePanel` currently has a `VerifyingMini` sub-component that displays "All features released" with a "Run UAT" button. This same UI renders whether the milestone is in `verifying` status (UAT pending) or `released` status (UAT passed, milestone shipped). A released milestone should celebrate completion and surface useful next actions rather than showing a verifying-state UI.

## Requirements

### Functional

1. **Victory banner** — When the milestone status is `released`, display a celebratory banner with a checkmark icon, the milestone title, and a "Released" label. The tone should be confident and understated — not flashy confetti, but a clear visual signal that this milestone shipped.

2. **Stats summary** — Show key completion stats:
   - Total features count
   - Total UAT runs completed
   - Latest UAT verdict (pass / pass_with_tasks / failed)
   - Date released (from the milestone's `updated_at` or latest passing UAT run's `completed_at`)

3. **Re-run UAT** — Provide a button to re-run UAT against the released milestone. This uses the same `useMilestoneUatRun` hook as the existing `VerifyingMini` component. When a UAT run is in progress, show a "Running" indicator instead.

4. **Submit manual UAT** — Include the "Submit manually" link that opens the `HumanUatModal`, same as the existing verifying UI.

5. **Next milestone link** — If another active milestone exists in the project, show a link to navigate to it. Use the project state's `milestones` array to find the next active milestone. If no active milestone exists, omit this section.

### Non-Functional

- The component must follow existing patterns: use `useSSE` for live updates, `api` client for data fetching, lucide-react for icons, and Tailwind CSS for styling.
- No new API endpoints required — all data is available from existing `getMilestone`, `listMilestoneUatRuns`, and `getProjectPrepare` endpoints.
- The component renders inside the `MilestoneDetail` page, conditionally shown when `milestone.status === 'released'`.

## Integration Point

The `MilestonePreparePanel` component (or the parent `MilestoneDetail` page) will route to `ReleasedPanel` when the milestone status is `released`, and continue showing the existing prepare/verifying UI for other statuses.

## Out of Scope

- Animations or confetti effects
- Milestone archival actions
- Historical milestone comparison
