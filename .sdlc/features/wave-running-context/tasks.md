# Tasks: Wave Running Context and Recovery Path

## T1: Add wave running context message to WavePlan

**File:** `frontend/src/components/features/WavePlan.tsx`

In the `WaveSection` component, add a static informational paragraph below the wave header row when `runWaveRunning === true`.

The message:
```
Agents are working — you don't need to stay here.
Results appear on this page when they're done.
```

Styling: `text-sm text-muted-foreground`, placed below the header button, outside the collapsible content area so it is always visible while the wave is running (not hidden by expand/collapse state).

The message disappears when `runWaveRunning` becomes `false`.

---

## T2: Add many-features-no-milestone recovery prompt to Dashboard

**File:** `frontend/src/pages/Dashboard.tsx`

Add a dismissible recovery prompt card that appears when:
- `orphanedActiveFeatures.length >= 5` (features not in any milestone, not archived, not released)
- `activeMilestones.length === 0`
- `recoveryPromptDismissed === false` (read from `localStorage`)

Rendered card:
```
You have N features without a milestone.

To run them in parallel waves, organize them into a milestone.
That's where Run Wave lives.

[ Organize into Milestone ]   [ × ]
```

Behavior:
- "Organize into Milestone" → `Link` to `/milestones`
- "×" dismiss button → sets `localStorage.setItem('sdlc_recovery_prompt_dismissed', 'true')` and hides the card
- Card disappears naturally when the user creates a milestone (condition `activeMilestones.length === 0` is no longer met)

Placement: Between the escalation section and the Wave Plan section (after the `PreparePanel` call).

---

## T3: (Stretch) First-wave-complete overlay

**Files:**
- `frontend/src/components/shared/WaveCompleteOverlay.tsx` (new)
- `frontend/src/App.tsx` or `frontend/src/components/layout/AppShell.tsx` (mount point)

Create a `WaveCompleteOverlay` component that:
- Listens for `run_finished` SSE events via `useAgentRuns()` context
- Triggers when a completed run has `run_type === 'milestone_run_wave'`
- Only triggers once per user lifetime (checks `localStorage.getItem('sdlc_first_wave_seen')`)
- Shows a dismissible bottom-right panel with:

```
Wave complete. N features built in parallel.

This is how SDLC works: you ponder, you commit, you run — then check in on results.
You don't need to watch while agents work.
```

- Dismiss (click `×` or anywhere on the panel): sets `localStorage.setItem('sdlc_first_wave_seen', 'true')`

Mount the component at the app root level so it is visible regardless of current page.
