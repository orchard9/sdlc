# Design: data-testid Attributes for React Components

## Approach

This is a pure annotation pass — no visual or behavioural changes. The design describes exactly which prop changes are needed in each file and why.

## Component Changes

### 1. `StatusBadge` — add `data-testid` prop support

`StatusBadge` is the shared component used everywhere for phase and status display. It currently accepts `status` and `className` only. We must add an optional `data-testid` passthrough so callers can brand individual badges without forking the component.

**Change:** Add `testId?: string` to `StatusBadgeProps` and render it as `data-testid={testId}` on the outer `<span>`.

```tsx
// Before
interface StatusBadgeProps {
  status: string
  className?: string
}

// After
interface StatusBadgeProps {
  status: string
  className?: string
  testId?: string
}

export function StatusBadge({ status, className, testId }: StatusBadgeProps) {
  ...
  return (
    <span
      data-testid={testId}
      className={cn(...)}
    >
      ...
    </span>
  )
}
```

### 2. `FeatureCard` — annotate phase badge and feature title

- `<h3>` element containing `{feature.title}` → `data-testid="feature-title"`
- `<StatusBadge status={feature.phase} />` → add `testId="phase-badge"`

### 3. `FeatureDetail` — annotate phase badge, feature title, next-action panel, artifact list, task list

- `<h2>` containing `{feature.title}` → `data-testid="feature-title"`
- `<StatusBadge status={feature.phase} />` → add `testId="phase-badge"`
- The conditional `<div>` wrapping the "Next action" content → `data-testid="next-action"`
- The `<section>` containing artifact viewers → `data-testid="artifact-list"`
- The `<section>` containing tasks → `data-testid="task-list"`

### 4. `ArtifactViewer` — annotate artifact status badge

- `<StatusBadge status={artifact.status} />` inside the header row → add `testId="artifact-status"`

### 5. `MilestoneDetail` — annotate milestone title and milestone status

- `<h2>` containing `{milestone.title}` → `data-testid="milestone-title"`
- `<StatusBadge status={milestone.status} />` → add `testId="milestone-status"`

### 6. `MilestonesPage` (`MilestoneCard`) — annotate milestone title and milestone status

- The `<Link>` element containing `{m.title}` → its parent or the Link itself gets `data-testid="milestone-title"` (wrap in a `<span>` to avoid putting testid on a router Link, or apply directly to the Link; both are fine for Playwright)
- `<StatusBadge status={m.status} />` → add `testId="milestone-status"`

## File Change Summary

| File | Lines changed |
|---|---|
| `frontend/src/components/shared/StatusBadge.tsx` | Add `testId` prop, spread to `data-testid` |
| `frontend/src/components/features/FeatureCard.tsx` | Add `data-testid` to title `<h3>` and `testId` to phase `StatusBadge` |
| `frontend/src/pages/FeatureDetail.tsx` | Add `data-testid` to title `<h2>`, phase `StatusBadge`, next-action div, artifact `<section>`, task `<section>` |
| `frontend/src/components/features/ArtifactViewer.tsx` | Add `testId` to status `StatusBadge` |
| `frontend/src/pages/MilestoneDetail.tsx` | Add `data-testid` to title `<h2>` and `testId` to status `StatusBadge` |
| `frontend/src/pages/MilestonesPage.tsx` | Add `data-testid` to title `<Link>` and `testId` to status `StatusBadge` in `MilestoneCard` |

## Verification

After all edits:
```bash
cd frontend && npx tsc --noEmit
```

Expected: zero errors.

## Risk

Minimal. `data-testid` attributes are ignored by React rendering, never affect layout, and are not read by any existing production code. TypeScript will verify the new prop at compile time.
