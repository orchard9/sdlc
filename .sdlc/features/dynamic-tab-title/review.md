# Code Review: Dynamic Browser Tab Title

## Changes

### `frontend/src/components/layout/AppShell.tsx`
- Added a `useEffect` that sets `document.title` to `{projectName} · {focus} · Ponder`
- For detail routes (2+ path segments), the slug is prepended: `slug · PageLabel`
- Dependencies: `[location.pathname, projectName]` — updates on every navigation and when project config loads

### `frontend/src/pages/HubPage.tsx`
- Added `useEffect(() => { document.title = 'Ponder Hub' }, [])` for hub mode

## Findings

### F1: Correct dependency arrays
Both `useEffect` hooks have correct dependency arrays. The AppShell effect re-runs on pathname change and projectName change. The HubPage effect runs once on mount. **No issue.**

### F2: No memory leaks
`document.title` assignment is synchronous — no cleanup needed. **No issue.**

### F3: TypeScript compiles cleanly
`npx tsc --noEmit` passes with zero errors. **No issue.**

### F4: Reuses existing infrastructure
The implementation reuses `titleFromPath()` and `projectName` — both already present in AppShell. No new state, no new API calls, no new dependencies. **Good.**

### F5: Edge case — root path
`/` splits to `[]` after filter, so `parts.length` is 0. Falls through to `base` only ("Dashboard"). Correct behavior.

### F6: Edge case — triple-segment paths like `/docs/section`
`/docs/section` → parts = `['docs', 'section']`, length = 2, so title becomes `section · Docs`. This is reasonable for all current routes.

## Verdict

**Approved.** Clean, minimal implementation. No issues found.
