# QA Plan: Pipeline Visibility Indicator

## Approach

Static visual inspection via `sdlc ui` + browser, plus TypeScript build validation. No automated Playwright tests required for this component (it is a stateless UI element).

## Build Verification

```bash
cd frontend && npm run build
```

Must complete with zero TypeScript errors and zero ESLint errors.

## Manual Checks

### 1. Component renders on Dashboard

- Open the Dashboard (`/`)
- Confirm the pipeline indicator is visible without scrolling
- Confirm five stage pills are rendered with arrow connectors between them

### 2. Stage pill content

- Each pill displays the correct label: Ponder, Plan, Commit, Run Wave, Ship
- Arrow connectors (`→`) appear between pills

### 3. Visual states

#### Empty project (no ponders, no milestones)
- Stage 0 (Ponder) is highlighted in primary color
- Stages 1–4 are ghost/outlined

#### Project with exploring ponders only
- Stage 0 (Ponder) is highlighted
- Stages 1–4 are ghost

#### Project with a committed ponder (or any milestone)
- Stage 1 (Plan) or Stage 2 (Commit) is highlighted accordingly
- Stages 0 and 1 before current show completed style (checkmark or dimmed)

#### Project with released milestone
- Stage 4 (Ship) is highlighted
- Stages 0–3 show completed style

### 4. Navigation

- Click each pill and confirm navigation to correct route:
  - Ponder → `/ponder`
  - Plan → `/ponder`
  - Commit → `/milestones`
  - Run Wave → `/milestones`
  - Ship → `/milestones`

### 5. Tooltip

- Hover over each pill and confirm the browser native tooltip (or shadcn Tooltip) appears with the correct description text

### 6. Responsive layout

- Resize browser to mobile width (~375px)
- Confirm indicator is visible and not broken (may wrap or shrink — just must not overflow)

## Acceptance Criteria (from spec)

- [ ] `PipelineIndicator` component renders on the Dashboard
- [ ] Five stages display as horizontal pills with arrows between them
- [ ] Current stage (furthest reached) is visually highlighted
- [ ] Each stage pill is clickable and navigates to the correct page
- [ ] Indicator is visible on page load without scrolling
- [ ] Tooltips appear on hover for each stage
- [ ] New project (no ponders, no milestones): Stage 1 (Ponder) is highlighted as the starting point
