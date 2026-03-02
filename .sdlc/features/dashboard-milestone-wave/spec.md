# Spec: Active Milestones and Run Wave on Dashboard

## Problem

Run Wave is buried 4+ navigation levels deep from the Dashboard: Dashboard → Milestones → Milestone Detail → Wave Plan → Run Wave button. A new user who doesn't know this path will never discover it without guidance.

The Dashboard currently shows zero-stats and a feature list but does not surface active milestones at all. "Run Wave" — the tool's defining capability — is invisible from the first screen.

## Solution

Add an "Active Milestones" section to the Dashboard that shows current milestone cards with their wave state and a direct "Run Wave" button.

## Implementation

### 1. Active Milestones Section on Dashboard

File: `frontend/src/pages/DashboardPage.tsx`

Add a new section below the stats bar (or as a primary section when milestones exist):

**Section heading:** "Active Milestones"

**Show condition:** At least one milestone with status `active` or `verifying` (i.e., not `released` or `cancelled`).

**Each milestone card shows:**
- Milestone title
- Milestone slug (subdued)
- Wave status: "Wave N ready — M features" or "Wave N running" or "Wave plan not yet generated"
- "Run Wave" button — visible and enabled when a wave is ready to run (wave plan exists, not all features done)
- "View Details" link → navigates to the milestone detail page

**"Run Wave" button behavior:** Navigates to the milestone detail page and triggers the run wave action (same as clicking "Run Wave" in the Wave Plan section). Alternatively, can navigate to `/milestones/:slug` — the milestone detail page can auto-scroll to the wave plan section.

### 2. Wave State Derivation

The Dashboard needs to know each milestone's wave state. This comes from the existing milestone data already available via the API:

- If milestone has `prepared: true` and has features in pending/implementation phase → wave is ready
- If milestone has active agent runs in progress → wave is running
- If milestone has no wave plan yet → show "Prepare milestone to generate wave plan" with a "Prepare" link

Use the existing SSE update pattern — Dashboard already receives project state updates. No new API endpoints needed if milestone list already includes feature phase counts.

### 3. Empty State Interaction

When no milestones exist (new project):
- Do NOT show "Active Milestones" section
- Show the identity empty state (handled by `dashboard-empty-state` feature)

When milestones exist but all are released:
- Show a "Completed Milestones" count or a simple "No active milestones — start a new ponder" prompt.

## Acceptance Criteria

- [ ] Dashboard shows "Active Milestones" section when at least one milestone is active
- [ ] Each milestone card displays title, wave number, and feature count
- [ ] "Run Wave" button is visible on a card when that milestone's wave is ready
- [ ] "Run Wave" button navigates to the milestone detail page (and ideally the wave plan section)
- [ ] Section does not appear when no milestones exist
- [ ] No new API endpoints required — uses existing milestone list data

## Out of Scope

- Full milestone management from Dashboard (create, edit, delete milestones)
- Wave execution logic (existing implementation unchanged)
- Milestone detail page changes (those stay as-is)
