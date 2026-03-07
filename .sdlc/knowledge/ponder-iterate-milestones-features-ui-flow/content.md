---
session: 1
timestamp: 2026-03-07T15:27:00Z
orientation:
  current: "Analysis complete — dead-ends identified across milestone detail + feature detail, implementation plan shaped"
  next: "Implement Phase 1: add MilestonePreparePanel to MilestoneDetail.tsx"
  commit: "Phase 1+2 implemented and visually verified — milestone detail has Run UAT button, feature detail has milestone breadcrumb"
---

**Xist · Owner**
iterate-milestones-features-ui-flow

http://localhost:51340/milestones/v46-realtime-activity-feed

This milestone is in a state where the UI doesn't know what to do next. Similar to the `iterate-ponder-ui-flow` project, iterate the ui flow for milestones and features.

---

**Facilitator**

Studied `MilestoneDetail.tsx`, `FeatureDetail.tsx`, `MilestonePreparePanel.tsx`, `MilestonesPage.tsx`, `useMilestoneUatRun.ts`, and the live UI at `localhost:51340`. Navigated the milestone detail page, milestones list, and dashboard to compare forward-motion patterns.

## The Dead-End Problem

The milestone detail page (`/milestones/v46-realtime-activity-feed`) shows:
- Title + "verifying" badge
- 3 features, all showing "released" with "done" action
- UAT History section saying "No UAT runs yet."
- **Nothing else. No forward action.**

Meanwhile, the milestones LIST page and the dashboard both have "All features released" + "Run UAT" buttons via `MilestonePreparePanel`. The component already exists — it just isn't used on the detail page.

## Page-by-Page Audit

| Page | Forward Motion | Dead-End? |
|------|---------------|-----------|
| **Dashboard** | Wave plan, Run UAT, progress bars, next commands | No |
| **Milestones list** | `MilestonePreparePanel` per milestone, Run UAT button | No |
| **Milestone detail** | Features list + UAT History... and nothing else | **YES** |
| **Feature detail (done)** | Green "Feature complete" badge, no links up | **YES** |
| **Feature detail (active)** | Next action + Run button + copy commands | No |

## Root Cause

Information flows down (dashboard → milestones → features) but never up. Feature detail doesn't know its parent milestone. Milestone detail doesn't embed the prepare/verify panel that already exists.

## Decisions

`⚑ Decided:` **Phase 1 — Add MilestonePreparePanel to MilestoneDetail.tsx.** Import the existing component and place it between the Features section and UAT History. This single change adds: wave plan during execution, "All features released" + Run UAT when verifying, progress bar, submit manually option. Zero new components needed.

`⚑ Decided:` **Phase 2a — Add milestone breadcrumb to FeatureDetail.tsx.** Derive milestone membership from `useProjectState().state.milestones` by scanning which milestone's `features[]` includes the current slug. Show as a link above the feature title.

`⚑ Decided:` **Phase 2b — Enhance "Feature complete" green badge.** When done + has parent milestone, add a "View milestone →" link.

`? Open:` **Phase 3 — Make feature pills clickable on milestones list page.** Currently they're just text badges. Low priority since you can expand to see `FeatureCard` components, but it would improve quick navigation.

`? Open:` **Feature → sibling features awareness.** When a feature is done, could show "3/3 features in {milestone} complete" progress. Nice but needs an extra API call or state derivation.

## Implementation Shape

### Phase 1 (primary fix): MilestoneDetail.tsx
```diff
+ import { MilestonePreparePanel } from '@/components/milestones/MilestonePreparePanel'

  // After Features section, before UAT History
+ <section className="mt-6">
+   <MilestonePreparePanel milestoneSlug={slug} />
+ </section>
```

### Phase 2a: FeatureDetail.tsx — milestone breadcrumb
```tsx
// Derive from project state
const parentMilestone = state?.milestones?.find(m => m.features.includes(slug))

// Render above title
{parentMilestone && (
  <Link to={`/milestones/${parentMilestone.slug}`}
    className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground mb-1">
    {parentMilestone.title || parentMilestone.slug} →
  </Link>
)}
```

### Phase 2b: FeatureDetail.tsx — done state enhancement
```tsx
{classification?.action === 'done' && parentMilestone && (
  <div className="flex items-center gap-2 px-3 py-2 bg-green-500/10 border border-green-500/30 rounded-xl mb-6">
    <span className="text-xs font-medium text-green-400">Feature complete</span>
    <Link to={`/milestones/${parentMilestone.slug}`}
      className="text-xs text-green-400 underline hover:text-green-300 ml-auto">
      View milestone →
    </Link>
  </div>
)}
```

## Files to Touch

1. `frontend/src/pages/MilestoneDetail.tsx` — add `MilestonePreparePanel` (1 import + 3 lines)
2. `frontend/src/pages/FeatureDetail.tsx` — add milestone breadcrumb + enhance done state (derive from `useProjectState`)

No new API endpoints. No new components. All pieces exist — they just need wiring.
