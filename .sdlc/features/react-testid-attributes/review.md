# Code Review: data-testid Attributes

## Summary

This change annotates six React component files with `data-testid` attributes for stable Playwright selector anchoring. No logic, styling, or behaviour is altered — this is a pure attribute-annotation pass.

## Files Changed

### `frontend/src/components/shared/StatusBadge.tsx`

**Change:** Added optional `testId?: string` prop to `StatusBadgeProps` and rendered it as `data-testid={testId}` on the root `<span>`.

**Review:** Correct approach. The `testId` prop is optional so all existing call sites continue to compile without modification. The attribute is only emitted when a `testId` is supplied (undefined attribute values are omitted by React). TypeScript enforces the type at every call site. No concerns.

### `frontend/src/components/features/FeatureCard.tsx`

**Changes:**
- `data-testid="feature-title"` on the `<h3>` title element.
- `testId="phase-badge"` on the phase `StatusBadge`.
- `data-testid="next-action"` on the next-action indicator div.

**Review:** All three elements are stable structural landmarks in the card. The `next-action` placement is the correct containing div — it wraps both the arrow icon and the action text, giving tests a single stable target. No concerns.

### `frontend/src/pages/FeatureDetail.tsx`

**Changes:**
- `data-testid="feature-title"` on the `<h2>` title heading.
- `testId="phase-badge"` on the phase `StatusBadge`.
- `data-testid="next-action"` on the directive panel `<div>` (conditional, only rendered when action ≠ done).
- `data-testid="artifact-list"` on the artifacts `<section>`.
- `data-testid="task-list"` on the tasks `<section>`.

**Review:** All placements are on the correct enclosing elements. The `next-action` div is conditionally rendered, which is correct — a Playwright test that expects `next-action` to be present only runs when the feature is in-progress, which is the intended behaviour. The `artifact-list` and `task-list` sections are the stable containers that wrap the respective lists. No concerns.

### `frontend/src/components/features/ArtifactViewer.tsx`

**Change:** `testId="artifact-status"` on the `<StatusBadge status={artifact.status} />` in the artifact header row.

**Review:** Correct element. This badge shows the current artifact status (missing, draft, approved, etc.) and is the element tests will most commonly query. No concerns.

### `frontend/src/pages/MilestoneDetail.tsx`

**Changes:**
- `data-testid="milestone-title"` on the `<h2>` milestone title heading.
- `testId="milestone-status"` on the `<StatusBadge status={milestone.status} />`.

**Review:** Both elements are stable, top-level landmarks. No concerns.

### `frontend/src/pages/MilestonesPage.tsx`

**Changes (inside `MilestoneCard`):**
- `data-testid="milestone-title"` on the `<Link>` rendering the milestone title.
- `testId="milestone-status"` on the `<StatusBadge status={m.status} />`.

**Review:** Applying `data-testid` directly to a React Router `<Link>` is valid — it renders as a standard `<a>` tag in the DOM and the attribute is passed through. Playwright's `locator('[data-testid="milestone-title"]')` will find it correctly. No concerns.

## What Was Not Changed

- `approve-button` / `reject-button` — no such buttons exist in the current UI; artifact approval is a CLI operation.
- `start-uat-button` — not present in `MilestoneDetail`.
- `directive-panel` — superseded by `next-action` (same semantic concept, better name).

## TypeScript

`npx tsc --noEmit` exits 0 with zero diagnostics. All new prop usages are type-safe.

## Verdict

**Approved.** The implementation exactly matches the spec and design. All targeted elements are correctly annotated, no existing behaviour is altered, and TypeScript is clean.
