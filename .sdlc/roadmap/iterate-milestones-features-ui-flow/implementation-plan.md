# Implementation Plan

## Phase 1: Milestone Detail — Add MilestonePreparePanel (HIGH IMPACT, LOW EFFORT)

**File:** `frontend/src/pages/MilestoneDetail.tsx`

Add `MilestonePreparePanel` between the Features section and the UAT History section. This single change gives the milestone detail page:
- Wave plan display during active execution
- "All features released" + "Run UAT" button when verifying
- Progress bar with released/total counts
- Submit manually option

```tsx
// After </section> for Features (~line 155), before UAT History section
<section className="mt-6">
  <MilestonePreparePanel milestoneSlug={slug} />
</section>
```

Import: `import { MilestonePreparePanel } from "@/components/milestones/MilestonePreparePanel"`

This is the highest-value change — it eliminates the primary dead-end.

## Phase 2: Feature Detail — Milestone Context

**File:** `frontend/src/pages/FeatureDetail.tsx`

### 2a. Show parent milestone breadcrumb

The feature object should include its milestone membership. Check if `feature.milestone` exists in the API response. If so, add a breadcrumb/link above the title:

```tsx
{feature.milestone && (
  <Link to={`/milestones/${feature.milestone}`}
    className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground mb-1">
    <FolderOpen className="w-3 h-3" />
    {feature.milestone}
  </Link>
)}
```

### 2b. Enhance "Feature complete" state

When a feature is done AND belongs to a milestone, show milestone progress context:

```tsx
{classification?.action === "done" && feature.milestone && (
  <div className="...">
    <span>Feature complete</span>
    <Link to={`/milestones/${feature.milestone}`}>
      View milestone →
    </Link>
  </div>
)}
```

## Phase 3: Milestone Detail — Feature Phase Badges

Currently features in the milestone detail use `FeatureCard` which already shows phase badges and next actions. This is actually already good — `FeatureCard` handles progress bars, phase badges, and run buttons. No change needed here.

## Phase 4: Milestones List — Feature Interactivity (STRETCH)

The milestones list shows feature slugs as non-interactive pills. Could make them clickable links:

```tsx
<Link to={`/features/${slug}`} className="...">
  {slug}
</Link>
```

## Files to Modify

| File | Change | Impact |
|------|--------|--------|
| `MilestoneDetail.tsx` | Add MilestonePreparePanel | Fixes primary dead-end |
| `FeatureDetail.tsx` | Add milestone breadcrumb + context | Shows forward motion from done features |
| `MilestonesPage.tsx` | Make feature pills clickable (stretch) | Better navigation |

## Dependencies

- Check if feature API response includes `milestone` field
- `MilestonePreparePanel` already handles all states via `useMilestoneUatRun` hook
- No new API endpoints needed