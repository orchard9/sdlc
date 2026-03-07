# Dead-End Analysis

## Milestone Detail Page (`/milestones/:slug`)

**Current state:** Shows features list + UAT History section. When all features are released (verifying state), the page dead-ends — no forward action visible.

**What exists but is NOT used here:**
- `MilestonePreparePanel` — has `VerifyingMini` component with "All features released" + "Run UAT" button
- `useMilestoneUatRun` hook — handles UAT run start/focus/modal
- Wave plan display during active execution

**The gap:** The milestones LIST page (`/milestones`) renders `MilestonePreparePanel` for each milestone, showing Run UAT. The DETAIL page does not.

## Feature Detail Page (`/features/:slug`)

**Current state:** When a feature is done, shows a green "Feature complete — no pending actions" badge. No forward motion.

**Missing:**
- No link back to parent milestone
- No "milestone context" — user doesn't know if this was the last feature or if others remain
- No suggestion like "All features in {milestone} complete — run UAT"

## Dashboard

**Current state:** Actually good — shows WAVE PLAN section with Run UAT button, milestone progress bars, next commands. But it's the only place with full forward motion.

## Milestones List Page

**Current state:** Shows features as pills + `MilestonePreparePanel` with Run UAT. Good forward motion but features are non-interactive (no progress info, no phase badges).

## Key Observations

1. **Information flows down, not up.** Dashboard knows about milestones and features. Milestone detail knows about features. Feature detail knows about... nothing above it.
2. **`MilestonePreparePanel` is the answer** for milestone detail — it already exists and handles all the verifying/wave states. Just needs to be added.
3. **Feature detail needs milestone context** — breadcrumb + "milestone sibling" awareness.