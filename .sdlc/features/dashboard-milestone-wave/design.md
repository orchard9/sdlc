# Design: Active Milestones and Run Wave on Dashboard

## Overview

Add wave state and a "Run Wave" button to the per-milestone sections already rendered on the Dashboard. The Dashboard already fetches and renders `state.milestones` — no new data fetching is needed. The `MilestonePreparePanel` component already provides the full wave plan UI including progress bar, wave list, and Run Wave button. We embed it inline per milestone card on the Dashboard.

## Current State

`Dashboard.tsx` already renders active milestones (lines 324-363). Each milestone section shows:
- Title, StatusBadge, slug, feature count
- A `CommandBlock` with `/sdlc-run <nextFeature>` or `/sdlc-milestone-verify`
- A grid of `FeatureCard` components

The Dashboard does **not** show wave state or a Run Wave button. "Run Wave" is only accessible via: Dashboard → Milestones page → MilestonePreparePanel per milestone card.

## Design

### ASCII Wireframe — Dashboard Milestone Section (after change)

```
┌─ Active Milestones ──────────────────────────────────────────────────────┐
│                                                                           │
│  v08-orchestrator-webhooks    [active]    v08-orchestrator-webhooks  4f  │
│  ────────────────────────────────────────────────────────────────────     │
│  [■■■■□□] 2/4 released  · wave 2  · 1 active                            │
│  ┌─ Wave 2 — Implementation ──────────────────────── 2 features  [Run Wave]─┐│
│  │  orchestrator-webhook-events   [implementation]   implement_task  [Run]  ││
│  │  orchestrator-sse-bridge       [specified]        create_design   [Run]  ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                           │
│  ┌ feature card ┐  ┌ feature card ┐  ┌ feature card ┐                   │
│  └──────────────┘  └──────────────┘  └──────────────┘                   │
└──────────────────────────────────────────────────────────────────────────┘
```

### Component Changes

#### `frontend/src/pages/Dashboard.tsx`

1. **Import `MilestonePreparePanel`** from `@/components/milestones/MilestonePreparePanel`.
2. **In the active milestones `.map()` block**, insert `<MilestonePreparePanel milestoneSlug={milestone.slug} />` between the heading row and the feature grid. Wrap it in a `<div className="mb-3">` to match surrounding spacing.
3. **Remove the existing `CommandBlock`** for individual milestones in the active section — the `MilestonePreparePanel` already provides equivalent or better actionability through its Run Wave button and wave item Run buttons. (The `/sdlc-milestone-verify` command block can stay for released milestones but active milestones will use the prepare panel.)

### No New Components

`MilestonePreparePanel` already handles:
- Calling `api.getProjectPrepare(milestoneSlug)` — the per-milestone prepare endpoint
- Subscribing to SSE `run_finished` events to refresh
- Rendering progress bar + wave plan + Run Wave button
- Showing nothing when no wave plan exists yet (returns `null`)
- Showing VerifyingMini when all features released

### Behavior Summary

| Scenario | Dashboard shows |
|---|---|
| No milestones | No active milestones section (existing behavior) |
| Milestones exist, no wave plan | Milestone header + feature grid only (panel returns null) |
| Wave plan ready | Milestone header + progress bar + wave plan with Run Wave button + feature grid |
| Wave running | Wave plan with "Running" button (focus to agent panel) |
| All features released | VerifyingMini with "Run UAT" button |

### Data Flow

```
ProjectState (SSE/polling)
  └── state.milestones[]          → milestone headers + feature grids (existing)

Per-milestone prepare endpoint (per MilestonePreparePanel)
  GET /api/prepare?milestone=<slug>
  └── PrepareResult               → progress bar + wave plan + Run Wave button
```

No new API endpoints. No new types. No changes to the server.

## Files Changed

| File | Change |
|---|---|
| `frontend/src/pages/Dashboard.tsx` | Import `MilestonePreparePanel`; embed it in active milestone sections; remove old CommandBlock for active milestones |

## Out of Scope

- Changes to MilestonePreparePanel, WavePlan, or any server-side code
- New API endpoints
- Changes to MilestoneDetail page
