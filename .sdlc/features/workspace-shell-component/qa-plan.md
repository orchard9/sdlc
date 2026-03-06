# QA Plan: WorkspaceShell component

## Verification Strategy

This is a pure structural refactor — no new user-facing behavior is introduced. QA focuses on:
1. TypeScript build passes with no errors
2. The four refactored pages render correctly (left pane, right pane, mobile stacking)
3. The `WorkspaceShell` component file exists and is used in all four pages

## Test Cases

### TC1: TypeScript build passes

**Steps:**
1. Run `cd frontend && npm run build`
2. Observe output

**Expected:** Build completes with zero TypeScript errors.

### TC2: PonderPage — desktop layout intact

**Steps:**
1. Navigate to `/ponder`
2. Observe layout at lg+ viewport

**Expected:** Left pane (w-72) and right pane both visible side-by-side.

### TC3: PonderPage — mobile stacking

**Steps:**
1. Navigate to `/ponder` at mobile viewport (< lg breakpoint)
2. Observe: left pane visible, right pane hidden
3. Select a ponder entry (navigate to `/ponder/:slug`)
4. Observe: right pane visible, left pane hidden

**Expected:** Mobile stacking behavior preserved.

### TC4: EvolvePage — desktop and mobile layout

Same as TC2/TC3 but for `/evolve` and `/evolve/:slug`.

### TC5: InvestigationPage — desktop and mobile layout

Same as TC2/TC3 but for `/investigations` and `/investigations/:slug`.

### TC6: GuidelinePage — desktop and mobile layout

Same as TC2/TC3 but for `/guidelines` and `/guidelines/:slug`.

### TC7: WorkspaceShell file exists and is referenced in all four pages

**Steps:**
1. Check that `frontend/src/components/layout/WorkspaceShell.tsx` exists
2. Grep for `WorkspaceShell` import in PonderPage, EvolvePage, InvestigationPage, GuidelinePage

**Expected:** File exists; all four pages import and use `WorkspaceShell`.

## Pass Criteria

All 7 test cases pass. Build is clean. No regressions in any of the four workspace pages.
