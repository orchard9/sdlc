# Spec: Feature Detail — Milestone Breadcrumb and Enhanced Done State

## Problem

The feature detail page is a dead-end. Users navigating to a feature have no way to see which milestone it belongs to or navigate back to the milestone detail page. Additionally, when a feature reaches `released` phase, the "done" indicator is a minimal green banner that provides no useful context about the completed journey.

## Solution

### 1. Milestone Breadcrumb

Add a breadcrumb bar at the top of the feature detail page showing the parent milestone context:

```
Milestones > [Milestone Title] > [Feature Title]
```

- "Milestones" links to the milestone list page
- "[Milestone Title]" links to the milestone detail page
- "[Feature Title]" is the current page (not linked)
- If the feature belongs to no milestone, show: `Features > [Feature Title]`

**API change:** The `/api/features/:slug` endpoint must include the parent milestone slug and title in the response. The server looks up all milestones and finds which one(s) contain this feature. Return the first match (features belong to at most one milestone in practice).

### 2. Enhanced Done State

When `classification.action === 'done'` (feature is in `released` phase), replace the minimal green banner with a richer completion panel:

- Green checkmark icon with "Released" label
- Show when the feature was released (last entry in `phase_history`)
- Show total phase count / journey duration (created_at → released_at)
- Link back to parent milestone if one exists

### 3. Archived Indicator

If `feature.archived === true`, show a muted "Archived" badge next to the phase badge. This is orthogonal to the done state — a feature can be archived at any phase.

## Out of Scope

- Editing milestone assignment from the feature detail page
- Showing feature position/order within the milestone
- Multiple milestone membership (not a real scenario today)

## Acceptance Criteria

1. Feature detail page shows a breadcrumb with clickable milestone link when the feature belongs to a milestone
2. Feature detail page shows "Features > Title" breadcrumb when no milestone owns the feature
3. Released features show an enhanced done panel with release date and journey duration
4. Archived features show an "Archived" badge
5. The `/api/features/:slug` response includes `milestone: { slug, title } | null`
