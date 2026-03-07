# Spec: Milestone Detail — MilestonePreparePanel Integration

## Problem

The `MilestoneDetail` page shows features and UAT history but lacks wave plan progress and the "Run UAT" action. The `MilestonePreparePanel` component already exists (`frontend/src/components/milestones/MilestonePreparePanel.tsx`) and renders wave progress, wave plan, and a verifying state with Run UAT button — but it is not mounted in `MilestoneDetail.tsx`.

## Solution

Add `MilestonePreparePanel` to the `MilestoneDetail` page between the header and the features list. It self-manages its data (calls `api.getProjectPrepare(milestoneSlug)`) and auto-refreshes via SSE, so integration is a single component mount.

## Requirements

1. Import and render `<MilestonePreparePanel milestoneSlug={slug} />` in `MilestoneDetail.tsx` after the header block and before the Features section.
2. The panel renders nothing when there are no waves and the milestone is not in verifying state (existing behavior), so there is no empty-state concern.
3. No new API endpoints, types, or backend changes needed — the component and API already exist.

## Out of Scope

- Changing `MilestonePreparePanel` internals.
- Modifying the dashboard `PreparePanel` (feature-level variant).
- Adding new data to the milestone detail API response.
