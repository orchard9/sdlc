# Spec: Wave Running Context and Recovery Path

## Problem

Two related gaps in the current wave experience:

**Gap 1 — The Watching Mode Trap:** When a wave runs, the UI shows a live log with active output. New users (Xist-type) assume they need to watch. The tool is designed for fire-and-forget, but it never says so. There's no indication that closing the tab is fine.

**Gap 2 — The Manual Feature Trap Recovery:** Users who start by creating features manually (before discovering waves) end up with 5–20+ features and no milestone. The UI does not detect this pattern or offer a recovery path. The user is stuck in "I have tasks, now what?" without a clear path to the Run Wave flow.

## Solution

Add a "you don't need to watch" contextual message during wave runs. Add a detection + recovery prompt when a user has many features but no milestone.

## Implementation

### 1. Wave Running Context Message

**Where:** In the Wave Plan section of the Milestone Detail page, when a wave is actively running.

**File:** `frontend/src/pages/MilestoneDetailPage.tsx` or `frontend/src/components/WavePlan.tsx` (wherever the active wave run state is rendered).

**Condition:** At least one agent run is active for the current milestone's wave features.

**Add below the live log / active run indicator:**

```
Agents are working — you don't need to stay here.
Results appear on this page when they're done.
```

Styling: `text-sm text-muted-foreground`, below the live log panel, not inside it. Not a dismissible toast — just static informational copy that appears when a wave is running and disappears when the wave completes.

**Do not add:** A "Come back later" button with no action. The copy itself is the affordance — it's permission language, not a button.

**First-wave marker (stretch):** Track in `localStorage` whether the user has seen their first completed wave. On the first completed wave, show an overlay or slide-in panel:

```
Wave complete. N features built in parallel.

This is how SDLC works: you ponder, you commit, you run — then check in on results.
You don't need to watch while agents work.
```

One-time display, dismissible, `localStorage` flag `sdlc_first_wave_seen: true` prevents repeat.

### 2. Many-Features-No-Milestone Recovery Prompt

**Where:** Dashboard, when a specific pattern is detected.

**Detection condition:** The user has 5 or more features in any non-released phase AND zero active milestones (no milestones, or all milestones are released).

**File:** `frontend/src/pages/DashboardPage.tsx`

**Add a notice card** (not a banner, not a warning — a helpful prompt) in the Dashboard content area:

```
You have N features without a milestone.

To run them in parallel waves, organize them into a milestone.
That's where Run Wave lives.

[ Organize into Milestone ]   [ Learn more ]
```

**"Organize into Milestone" button behavior:**
- Navigate to `/milestones` (Milestone list page) with a query param that opens the "Create Milestone" form: `/milestones?create=1`
- Or directly to `/milestones/new` if that route exists
- The user then creates a milestone and adds features to it via the existing `sdlc milestone add-feature` equivalent in the UI

**"Learn more" link:** Links to a docs page or shows an inline explainer. If docs don't exist yet, omit this link — the button is sufficient.

**Dismissibility:** The prompt can be dismissed with an X button. Store dismissal in `localStorage` (`sdlc_recovery_prompt_dismissed: true`). If the user creates a milestone, the prompt disappears naturally (condition no longer met).

**Threshold:** 5 features is the trigger. This avoids showing the prompt to users who just created 1–2 features while exploring. Adjust threshold based on feedback.

## Acceptance Criteria

- [ ] When a wave is actively running, "Agents are working — you don't need to stay here. Results appear on this page when they're done." is visible below the live log
- [ ] The message disappears when the wave completes
- [ ] When a user has 5+ features and no active milestones, the recovery prompt appears on Dashboard
- [ ] Recovery prompt "Organize into Milestone" navigates to the milestone creation flow
- [ ] Recovery prompt is dismissible (persisted via localStorage)
- [ ] Recovery prompt disappears naturally when user creates a milestone
- [ ] (Stretch) First-wave-complete overlay appears once and is suppressed on subsequent waves via localStorage flag

## Out of Scope

- Automated milestone creation from existing features (that's a complex agent-side feature)
- Persistent "watching mode" toggle or setting
- Push notifications when waves complete
