# Task Breakdown: data-testid Attributes

## Task 1 — Extend StatusBadge with testId prop

**File:** `frontend/src/components/shared/StatusBadge.tsx`

Add optional `testId?: string` to the `StatusBadgeProps` interface and render it as `data-testid={testId}` on the root `<span>`. This is a prerequisite for all badge annotations.

---

## Task 2 — Annotate FeatureCard

**File:** `frontend/src/components/features/FeatureCard.tsx`

- Add `data-testid="feature-title"` to the `<h3>` that renders `{feature.title}`.
- Add `testId="phase-badge"` to the `<StatusBadge status={feature.phase} />`.

---

## Task 3 — Annotate FeatureDetail

**File:** `frontend/src/pages/FeatureDetail.tsx`

- Add `data-testid="feature-title"` to the `<h2>` rendering `{feature.title}`.
- Add `testId="phase-badge"` to `<StatusBadge status={feature.phase} />`.
- Add `data-testid="next-action"` to the outer `<div>` of the next-action conditional block.
- Add `data-testid="artifact-list"` to the `<section>` wrapping artifact viewers.
- Add `data-testid="task-list"` to the `<section>` wrapping the tasks list.

---

## Task 4 — Annotate ArtifactViewer

**File:** `frontend/src/components/features/ArtifactViewer.tsx`

- Add `testId="artifact-status"` to the `<StatusBadge status={artifact.status} />` in the header row.

---

## Task 5 — Annotate MilestoneDetail

**File:** `frontend/src/pages/MilestoneDetail.tsx`

- Add `data-testid="milestone-title"` to the `<h2>` rendering `{milestone.title}`.
- Add `testId="milestone-status"` to `<StatusBadge status={milestone.status} />`.

---

## Task 6 — Annotate MilestonesPage

**File:** `frontend/src/pages/MilestonesPage.tsx`

- In `MilestoneCard`: add `data-testid="milestone-title"` to the `<Link>` rendering `{m.title}`.
- In `MilestoneCard`: add `testId="milestone-status"` to `<StatusBadge status={m.status} />`.

---

## Task 7 — TypeScript verification

Run `cd frontend && npx tsc --noEmit` and confirm zero errors. Fix any type issues found before marking complete.
