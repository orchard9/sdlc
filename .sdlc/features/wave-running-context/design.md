# Design: Wave Running Context and Recovery Path

## Overview

Two small UI additions to reduce two user-experience friction points:

1. **Wave Running Context Message** — a static informational strip shown in `WavePlan.tsx` while a wave run is active, reassuring the user they don't need to watch.
2. **Many-Features-No-Milestone Recovery Prompt** — a dismissible notice card in `Dashboard.tsx` that appears when a user has 5+ features and no active milestones, offering a path to milestone creation.

Both changes are purely frontend. No backend changes are needed.

---

## Part 1: Wave Running Context Message

### Location

`frontend/src/components/features/WavePlan.tsx` — inside `WaveSection`, below the wave header row, visible only when `runWaveRunning === true`.

### Condition

`runWaveRunning` is already derived in `WaveSection` from `useAgentRuns().isRunning(runWaveKey)`. The message appears when this is `true` and disappears when it becomes `false` (wave completes).

### Rendered Output

```
Agents are working — you don't need to stay here.
Results appear on this page when they're done.
```

Styling: `text-sm text-muted-foreground` paragraph, placed below the wave expand/collapse header (inside the collapsible content area, at the bottom), or as a static band just below the header row outside the collapsible — whichever is less visually crowded. Static, non-dismissible.

### Stretch: First-Wave-Complete Overlay

On the `run_finished` SSE event (from `AgentRunContext`), if the completed run has `run_type === 'milestone_run_wave'` and `localStorage.getItem('sdlc_first_wave_seen')` is falsy, show a one-time slide-in panel (bottom-right, similar in style to a toast but larger and dismissible):

```
Wave complete. N features built in parallel.

This is how SDLC works: you ponder, you commit, you run — then check in on results.
You don't need to watch while agents work.
```

Dismiss: clicking anywhere on the panel or an explicit `×` button sets `localStorage.setItem('sdlc_first_wave_seen', 'true')`.

Implementation note: this overlay is a separate React component `WaveCompleteOverlay` rendered at the `AppShell` or `App` level so it's visible regardless of what page the user is on when the wave finishes.

---

## Part 2: Many-Features-No-Milestone Recovery Prompt

### Location

`frontend/src/pages/Dashboard.tsx` — rendered as a card in the main content area, positioned between the "Needs Your Attention" section and the Wave Plan section. The same area that currently shows `setupIncomplete` and escalation cards.

### Detection Condition

```ts
const orphanedActiveFeatures = state.features.filter(
  f => !f.archived && !assignedSlugs.has(f.slug) && f.phase !== 'released'
)
const noActiveMilestones = activeMilestones.length === 0
const showRecoveryPrompt =
  orphanedActiveFeatures.length >= 5 &&
  noActiveMilestones &&
  !recoveryPromptDismissed
```

`recoveryPromptDismissed` is read from `localStorage` via `useState` initialized from `localStorage.getItem('sdlc_recovery_prompt_dismissed') === 'true'`.

The condition uses `orphanedActiveFeatures` (features not assigned to any milestone, not archived, not released) because `ungrouped` is already computed in `Dashboard.tsx` for this purpose.

### Rendered Output

```
You have N features without a milestone.

To run them in parallel waves, organize them into a milestone.
That's where Run Wave lives.

[ Organize into Milestone ]   [ ×  Dismiss ]
```

Styling: `bg-primary/5 border border-primary/20 rounded-xl p-4 mb-6` — calm, informational, not alarming. Mirror the pattern of the setup incomplete card but with primary tones instead of amber.

**"Organize into Milestone" button:** `Link` to `/milestones` — the Milestones page. No query param needed; the user creates a milestone from there. The prompt disappears naturally once the user creates a milestone (condition `activeMilestones.length === 0` is no longer met).

**Dismiss button:** Small `×` button top-right of the card. On click, sets `localStorage.setItem('sdlc_recovery_prompt_dismissed', 'true')` and calls `setRecoveryPromptDismissed(true)`.

---

## Component Structure

### New / Modified Files

| File | Change |
|---|---|
| `frontend/src/components/features/WavePlan.tsx` | Add running context message in `WaveSection` |
| `frontend/src/pages/Dashboard.tsx` | Add recovery prompt card |
| `frontend/src/components/shared/WaveCompleteOverlay.tsx` | New component (stretch — first-wave overlay) |
| `frontend/src/App.tsx` or `AppShell.tsx` | Mount `WaveCompleteOverlay` at root level (stretch) |

### No Backend Changes

All state is derived from existing `ProjectState` (features, milestones) and `AgentRunContext` (isRunning). No new API endpoints, no new SSE events required for the core feature. The stretch overlay listens to the existing `run_finished` SSE event.

---

## ASCII Wireframe

### Wave Running Context Message

```
┌──────────────────────────────────────────────────┐
│ ▼ Wave 1 — Foundation     3 features   [Running] │  ← existing header
│                                                  │
│   Agents are working — you don't need to         │  ← NEW (visible when running)
│   stay here. Results appear on this page         │
│   when they're done.                             │
│                                                  │
│   feature-a  [specified]  implement_task  [···]  │  ← existing feature rows
│   feature-b  [draft]      create_spec     [···]  │
└──────────────────────────────────────────────────┘
```

### Recovery Prompt Card

```
┌──────────────────────────────────────────────────┐
│ You have 7 features without a milestone.       × │
│                                                  │
│ To run them in parallel waves, organize them     │
│ into a milestone. That's where Run Wave lives.   │
│                                                  │
│ [ Organize into Milestone ]                      │
└──────────────────────────────────────────────────┘
```

---

## Acceptance Criteria (from spec)

- When a wave is actively running, the context message is visible in the wave section
- The message disappears when the wave completes
- When a user has 5+ features without a milestone and no active milestones, the recovery prompt appears on Dashboard
- "Organize into Milestone" navigates to `/milestones`
- Recovery prompt is dismissible (persisted via `localStorage`)
- Recovery prompt disappears naturally when user creates a milestone (condition no longer met)
- (Stretch) First-wave-complete overlay appears once and is suppressed on subsequent waves via `localStorage` flag `sdlc_first_wave_seen`
