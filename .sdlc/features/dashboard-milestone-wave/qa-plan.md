# QA Plan: Active Milestones and Run Wave on Dashboard

## Scope

Verify that the Dashboard correctly embeds `MilestonePreparePanel` per active milestone, surfacing wave state and a Run Wave button. No server changes; all tests are frontend-only.

## Manual Test Scenarios

### Scenario 1 — Milestone with wave plan ready

**Setup:** Project has at least one active milestone with a generated wave plan (at least one feature not yet released).

**Steps:**
1. Open the Dashboard (`/`).
2. Locate the active milestone section.

**Expected:**
- Progress bar visible (e.g., "1/3 released · wave 2").
- Wave plan accordion visible with at least one wave row.
- Wave 1 row has a "Run Wave" button.
- No stale CommandBlock (`/sdlc-run <slug>`) above the feature grid for that milestone.

### Scenario 2 — Milestone with no wave plan yet

**Setup:** Active milestone exists but `sdlc prepare` has not been run.

**Steps:**
1. Open the Dashboard.
2. Locate the active milestone section.

**Expected:**
- `MilestonePreparePanel` renders nothing (returns `null` when no wave plan exists).
- Milestone heading, slug, feature count, and feature grid still render normally.
- No empty boxes or broken layout.

### Scenario 3 — All milestone features released (verifying state)

**Setup:** Active milestone where all features are in `released` phase.

**Steps:**
1. Open the Dashboard.

**Expected:**
- `VerifyingMini` component renders showing "All features released" with a "Run UAT" button.
- No wave plan accordion shown.

### Scenario 4 — No active milestones

**Setup:** Project has no milestones, or all milestones are `released`.

**Steps:**
1. Open the Dashboard.

**Expected:**
- No "Active Milestones" section renders (existing empty-state behavior preserved).
- No JavaScript errors in console.

### Scenario 5 — Run Wave button triggers agent run

**Setup:** Active milestone with a ready wave plan.

**Steps:**
1. Click "Run Wave" on Wave 1 in the Dashboard.

**Expected:**
- Agent panel opens showing the run-wave run in progress.
- "Run Wave" button changes to "Running" state.
- No navigation away from the Dashboard occurs.

### Scenario 6 — SSE refresh updates wave state

**Setup:** Wave plan result changes server-side (e.g., a feature advances phase).

**Steps:**
1. Keep Dashboard open.
2. Trigger a feature state change via CLI or another tab.

**Expected:**
- `MilestonePreparePanel` refreshes automatically via SSE `run_finished` event.
- Progress bar and wave item list update without full page reload.

## TypeScript / Build Check

- `npm run build` in `frontend/` must complete with zero type errors.
- No new ESLint warnings introduced.

## Acceptance Criteria (from spec)

- [ ] Dashboard shows wave plan section per active milestone when wave plan exists.
- [ ] "Run Wave" button is visible and functional when wave is ready.
- [ ] No CommandBlock for active milestones (replaced by MilestonePreparePanel).
- [ ] Section absent when no milestones exist.
- [ ] No new API endpoints required.
- [ ] TypeScript build clean.
