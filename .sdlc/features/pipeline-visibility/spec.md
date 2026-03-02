# Spec: Pipeline Visibility Indicator

## Problem

SDLC has a clear five-stage flow: Ponder → Plan → Commit → Run Wave → Ship. This flow is never visible in the UI. New users have no mental map of where they are or where they're going.

Maya's observation: "This single UI element would have told Xist everything he needed to know. He would have seen 'I'm in Ponder stage. There are four more stages. Run Wave is what I'm moving toward.'"

## Solution

Add a persistent horizontal pipeline indicator to the Dashboard (and optionally the Milestones page) that shows the five stages, highlights the current stage, and makes stages clickable to navigate.

## Implementation

### 1. PipelineIndicator Component

**File:** `frontend/src/components/PipelineIndicator.tsx` (new component)

**Stages (in order):**
1. Ponder → `/ponder`
2. Plan → `/ponder` (same page, different state — ponders that are committed become milestones)
3. Commit → `/milestones` (the commit action creates a milestone from a ponder)
4. Run Wave → `/milestones` (the run wave action is on the milestone detail page)
5. Ship → `/milestones` (released milestones)

**Visual design:**
- Horizontal pill row: `[ Ponder ] → [ Plan ] → [ Commit ] → [ Run Wave ] → [ Ship ]`
- Current stage: filled/highlighted pill (primary color)
- Completed stages: filled/dimmed or with a checkmark
- Future stages: outlined/ghost pills
- Arrow connectors between pills (→)
- Compact: fits in a ~60px tall horizontal bar

**Stage determination logic (stateless, derived from project data):**
- Stage 1 (Ponder): at least one ponder exists with status `exploring` or `complete`
- Stage 2 (Plan): at least one ponder exists with status `committed` OR at least one milestone exists
- Stage 3 (Commit): at least one milestone exists
- Stage 4 (Run Wave): at least one milestone exists with `prepared: true` or active wave runs
- Stage 5 (Ship): at least one milestone with status `released`

**Current stage:** the highest stage that has been reached (greedy — show the furthest progress).

**Clickability:** Each pill is a link to its destination page. Clicking "Run Wave" goes to `/milestones` (user can select a milestone from there).

### 2. Placement on Dashboard

**File:** `frontend/src/pages/DashboardPage.tsx`

Place the `PipelineIndicator` component:
- Below the page header / project title area
- Above the stats bar and main content
- Full-width or constrained to content width (matches existing layout)

It should be visible immediately when the Dashboard loads, without scrolling.

### 3. Placement on Milestones Page (Optional, Stretch)

If the Milestones page has a header area, add `PipelineIndicator` there too. This reinforces the flow when the user is in the milestone management phase.

### 4. Tooltip Context on Hover

Each stage pill shows a tooltip on hover explaining what that stage means:

- Ponder: "Explore ideas before committing to a plan"
- Plan: "Review and refine the auto-generated milestone plan"
- Commit: "Commit the plan — creates features in wave order"
- Run Wave: "Start a wave — agents build features in parallel"
- Ship: "Features shipped — milestone complete"

Tooltips use the existing Tooltip component from the project's component library (shadcn/ui `Tooltip` or equivalent).

## Acceptance Criteria

- [ ] `PipelineIndicator` component renders on the Dashboard
- [ ] Five stages display as horizontal pills with arrows between them
- [ ] Current stage (furthest reached) is visually highlighted
- [ ] Each stage pill is clickable and navigates to the correct page
- [ ] Indicator is visible on page load without scrolling
- [ ] Tooltips appear on hover for each stage
- [ ] New project (no ponders, no milestones): Stage 1 (Ponder) is highlighted as the starting point

## Out of Scope

- Per-milestone pipeline state (showing which milestone is at which stage)
- Animation or transitions between stages
- Embeddeding the indicator in every page (Dashboard is sufficient for v1)
