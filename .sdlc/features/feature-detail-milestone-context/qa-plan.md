# QA Plan: Feature Detail — Milestone Breadcrumb and Enhanced Done State

## Test Strategy

Manual verification against the running dev server (`localhost:7777`).

## Test Cases

### TC1: Breadcrumb — feature with milestone
1. Navigate to a feature that belongs to a milestone
2. Verify breadcrumb shows: `Milestones / [Milestone Title] / [Feature Title]`
3. Click milestone title — navigates to milestone detail page
4. Click "Milestones" — navigates to milestone list

### TC2: Breadcrumb — feature without milestone
1. Navigate to a feature that does not belong to any milestone
2. Verify breadcrumb shows: `Features / [Feature Title]`
3. Click "Features" — navigates to feature list (dashboard)

### TC3: API returns milestone field
1. Call `GET /api/features/<slug-with-milestone>`
2. Verify response contains `"milestone": { "slug": "...", "title": "..." }`
3. Call `GET /api/features/<slug-without-milestone>`
4. Verify response contains `"milestone": null`

### TC4: Enhanced done panel
1. Navigate to a feature in `released` phase
2. Verify done panel shows green checkmark, "Released" label
3. Verify release date is displayed
4. Verify journey duration is displayed
5. If feature has a milestone, verify milestone link is present and clickable

### TC5: Archived badge
1. Navigate to a feature where `archived: true`
2. Verify "Archived" badge appears next to the phase badge
3. Navigate to a non-archived feature — badge absent

### TC6: Regression — error/loading states
1. Verify loading skeleton still appears while feature loads
2. Verify error state still renders correctly for corrupt features

## Pass Criteria

All test cases pass. No regressions in existing feature detail functionality.
