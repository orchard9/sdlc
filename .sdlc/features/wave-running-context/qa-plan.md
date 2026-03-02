# QA Plan: Wave Running Context and Recovery Path

## Approach

Manual visual inspection and interaction testing in the running UI. No new Playwright tests required — this feature is tested via two simple observational scenarios.

---

## Test Cases

### TC1: Wave Running Context Message — appears and disappears

**Precondition:** A milestone with a wave plan exists and at least one wave is configured.

**Steps:**
1. Navigate to the Milestones page and open a milestone that has a wave plan.
2. Click "Run Wave" on Wave 1.
3. Observe the wave section immediately after clicking.

**Expected:**
- The message "Agents are working — you don't need to stay here. Results appear on this page when they're done." appears below the wave header.
- The message is styled as `text-sm text-muted-foreground` — subdued, not alarming.
- The message is NOT inside the collapsible feature list — it should be visible regardless of expand/collapse state.

**When the run finishes:**
- The message disappears.

**Pass criteria:** Message appears when running, disappears when done. No JS errors.

---

### TC2: Many-Features-No-Milestone Recovery Prompt — appears, links correctly, and dismisses

**Precondition:** The project has 5 or more features in non-released, non-archived phases, and zero active (non-released) milestones. The `sdlc_recovery_prompt_dismissed` localStorage key is not set.

**Steps:**
1. Navigate to Dashboard.
2. Observe the recovery prompt.

**Expected:**
- Card appears with "You have N features without a milestone." (correct feature count).
- Copy explains milestones and Run Wave.
- "Organize into Milestone" button is a link to `/milestones`.
- "×" dismiss button is visible.

**Steps — dismiss:**
3. Click "×".
4. Reload the page.

**Expected after dismiss:**
- Card is hidden immediately on click.
- After reload, card remains hidden (persisted via localStorage).

**Steps — natural disappearance:**
5. Remove the `sdlc_recovery_prompt_dismissed` key from localStorage (DevTools > Application > Local Storage).
6. Create a milestone via `sdlc milestone create ...`.
7. Reload Dashboard.

**Expected:**
- Card is no longer shown (condition `activeMilestones.length === 0` is no longer met).

**Pass criteria:** Card appears when condition is met, links correctly, dismisses and stays dismissed. Disappears naturally when milestone exists.

---

### TC3: Recovery Prompt — does NOT appear when threshold is not met

**Precondition:** The project has fewer than 5 ungrouped active features.

**Steps:**
1. Navigate to Dashboard.

**Expected:**
- Recovery prompt card is NOT visible.

**Pass criteria:** No false positives for users with fewer than 5 orphaned features.

---

### TC4: (Stretch) First-Wave-Complete Overlay — appears once

**Precondition:** `sdlc_first_wave_seen` is not set in localStorage. A wave run completes.

**Steps:**
1. Clear `sdlc_first_wave_seen` from localStorage.
2. Start and let a wave run complete.
3. Observe the UI.

**Expected:**
- A dismissible overlay/slide-in panel appears with "Wave complete. N features built in parallel." copy.
- Clicking "×" or the panel dismisses it.
- After dismissal, running another wave does NOT show the overlay again.

**Pass criteria:** Overlay appears exactly once, dismissed state persists.

---

## Automated Testing

No new Playwright E2E tests. The feature is inherently state-dependent (requires an active run or specific feature counts) and is covered by manual inspection. If the project gains a mock-state harness in the future, TC1 and TC2 would be excellent candidates for automation.

---

## Non-Goals

- Pixel-perfect visual tests
- Testing localStorage persistence across browser sessions (beyond single-session reload)
- Load or performance testing
