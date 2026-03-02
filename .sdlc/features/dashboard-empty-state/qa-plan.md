# QA Plan: Dashboard Empty State Redesign

## Approach

Manual inspection of the compiled frontend + grep-based verification of string changes. No new
automated tests required for this scope; changes are purely cosmetic/UX.

## Test Cases

### TC1 — Amber banner is gone

**Precondition:** Dashboard renders (any project state).
**Steps:** Load the Dashboard page.
**Expected:** No amber/yellow "Project setup is incomplete" or "agents need more context" banner is
visible above the project title.
**Pass criteria:** Zero amber banner elements in the rendered DOM.

### TC2 — Empty state renders for zero-content projects

**Precondition:** Project has 0 milestones and 0 features.
**Steps:** Load the Dashboard page.
**Expected:**
- Identity headline visible: "SDLC turns ideas into shipped software."
- Tagline visible: "Describe what you're building — agents will build it in parallel waves."
- "New Ponder" button present with primary styling.
- No "No features yet." text.
- No `sdlc feature create` CLI hint.
**Pass criteria:** All three elements present, old empty state absent.

### TC3 — "New Ponder" button navigates to /ponder

**Precondition:** Empty-state is visible (TC2 conditions).
**Steps:** Click the "New Ponder" button.
**Expected:** Browser navigates to `/ponder`.
**Pass criteria:** URL changes to `/ponder`.

### TC4 — Normal dashboard for projects with content

**Precondition:** Project has at least one milestone or one feature.
**Steps:** Load the Dashboard page.
**Expected:**
- `DashboardEmptyState` is NOT rendered.
- Normal feature cards and/or milestone sections are visible.
**Pass criteria:** Empty-state component absent from DOM.

### TC5 — "setup incomplete" strings replaced

**Precondition:** Codebase search.
**Steps:** Run a case-insensitive grep for "setup incomplete" across `frontend/src/`.
**Expected:** Zero matches.
**Pass criteria:** `grep -ri "setup incomplete" frontend/src/` returns no results.

### TC6 — No TypeScript / lint errors

**Steps:** Run `npm run build` (or `npx tsc --noEmit`) in the `frontend/` directory.
**Expected:** Build completes with no errors.
**Pass criteria:** Exit code 0.

## Verification Commands

```bash
# TC5 — no "setup incomplete" strings
grep -ri "setup incomplete" frontend/src/

# TC6 — TypeScript clean
cd frontend && npm run build
```
