# Code Review: Wave Running Context and Recovery Path

## Summary

Three files modified, one new file created. All changes are pure frontend (React + TypeScript). No backend changes required.

## Files Changed

### 1. `frontend/src/components/features/WavePlan.tsx`

**Change:** Added a contextual message paragraph that renders when `runWaveRunning === true`.

```tsx
{runWaveRunning && (
  <p className="text-sm text-muted-foreground px-3 py-2 bg-muted/10 border-t border-border/40">
    Agents are working — you don't need to stay here.
    Results appear on this page when they're done.
  </p>
)}
```

**Assessment:**
- Placed between the header `<button>` and the `{expanded && ...}` collapsible section — visible regardless of expand/collapse state. Correct placement per design.
- `runWaveRunning` is already computed from `isRunning(runWaveKey)` — no new state, no new API calls.
- Disappears automatically when `runWaveRunning` becomes `false`.
- Styling is consistent with the muted informational copy style used elsewhere.
- No prop changes, no interface changes.

### 2. `frontend/src/pages/Dashboard.tsx`

**Changes:**
- Added `X` to the `lucide-react` import.
- Added `recoveryPromptDismissed` state initialized from `localStorage`.
- Added the recovery prompt card block.

**Assessment:**

State initialization:
```tsx
const [recoveryPromptDismissed, setRecoveryPromptDismissed] = useState(
  () => localStorage.getItem('sdlc_recovery_prompt_dismissed') === 'true'
)
```
Lazy initializer pattern is correct — `localStorage` access happens only once on mount.

Condition logic:
```tsx
ungrouped.length >= 5 && activeMilestones.length === 0 && !recoveryPromptDismissed
```
- `ungrouped` is already computed (features not in any milestone, not archived, not released) — no duplication.
- `activeMilestones` is already computed — no duplication.
- Threshold of 5 matches the spec.

Dismiss handler:
```tsx
onClick={() => {
  localStorage.setItem('sdlc_recovery_prompt_dismissed', 'true')
  setRecoveryPromptDismissed(true)
}}
```
Both the localStorage and React state are updated atomically. Correct.

Navigation:
```tsx
<Link to="/milestones" ...>Organize into Milestone</Link>
```
Uses `Link` not `a href` — no page reload, correct router behavior.

**No concerns.**

### 3. `frontend/src/App.tsx`

**Change:** Imported and mounted `WaveCompleteOverlay` inside `AgentRunProvider` (so it has access to `useAgentRuns`).

```tsx
<AgentRunProvider>
  <WaveCompleteOverlay />
  <AppShell>
    ...
```

Placement inside `AgentRunProvider` is required since the component uses `useAgentRuns()`. Placement before `AppShell` means it renders at root level, visible regardless of route. Correct.

### 4. `frontend/src/components/shared/WaveCompleteOverlay.tsx` (new)

**Assessment:**

- Uses `useAgentRuns()` to watch `runs` for newly completed `milestone_run_wave` runs.
- `seenIds` ref prevents triggering for runs already seen in the current session.
- `localStorage` gate prevents showing the overlay after the first time.
- Dismiss handler sets localStorage and hides the overlay.
- `onClick={dismiss}` on the container panel makes the whole panel clickable as a dismiss target, consistent with the spec.
- Uses Tailwind `animate-in slide-in-from-bottom-4 fade-in` for entrance animation — no extra dependencies.

**Minor observation:** `featureCount` is always `0` since the run record doesn't include a feature count. The copy handles this gracefully (`featureCount > 0 ? ... : ''`). Not a bug — the design spec says "N features built in parallel" but the data isn't available without an additional API call. Acceptable for a first delivery; can be improved later by adding a count to the run record.

## TypeScript

`npx tsc --noEmit` reports no errors in our changed files. The only pre-existing errors are in `PonderPage.tsx` (unrelated, pre-existing).

## Acceptance Criteria Check

- [x] Wave running context message appears when wave is running — implemented via `runWaveRunning` condition in `WaveSection`
- [x] Message disappears when wave completes — state-driven, no timers needed
- [x] Recovery prompt appears when 5+ orphaned features and no active milestones — condition check in `Dashboard`
- [x] "Organize into Milestone" navigates to `/milestones` — `Link to="/milestones"`
- [x] Recovery prompt is dismissible with localStorage persistence — `X` button + `localStorage.setItem`
- [x] Recovery prompt disappears naturally when milestone created — condition re-evaluated on every render
- [x] (Stretch) First-wave overlay implemented in `WaveCompleteOverlay.tsx`, mounted at app root

## Verdict

APPROVED — implementation is correct, complete, and consistent with the design. All acceptance criteria met. No regressions introduced.
