# QA Plan: Fix /actions Page Black Screen

## Scope

Verify that the `/actions` page renders correctly after the PATH_LABELS fix and frontend rebuild.

## Test Cases

### TC1 — Direct navigation renders page content
1. Navigate directly to `http://localhost:<port>/actions` in a fresh browser tab
2. **Expected:** Actions page renders with three sections: "Scheduled Actions", "Webhook Routes", "Webhook Events"
3. **Expected:** No `No routes matched location "/actions"` warning in browser console
4. **Fail if:** `<main>` is empty or a spinner runs indefinitely

### TC2 — Sidebar link is present and navigates correctly
1. Load any page (e.g., Dashboard)
2. Look in the "setup" section of the sidebar
3. **Expected:** "Actions" link with CalendarClock icon is visible
4. Click the "Actions" link
5. **Expected:** URL changes to `/actions`, Actions page renders
6. **Expected:** The "Actions" link is highlighted/active

### TC3 — Mobile header title (PATH_LABELS fix)
1. Set viewport to mobile size (e.g., 390px wide)
2. Navigate to `/actions`
3. **Expected:** Mobile header shows "Actions" as the page title (not "SDLC")

### TC4 — BottomTabBar Setup tab activates
1. On mobile viewport, navigate to `/actions`
2. **Expected:** The "Setup" tab in the BottomTabBar is highlighted/active

### TC5 — DB unavailable warning (graceful degradation)
1. Navigate to `/actions` when the orchestrator daemon is not running
2. **Expected:** Page renders (not a black screen); shows a yellow warning banner "Orchestrator DB unavailable"
3. **Expected:** Scheduled Actions and Webhook sections show empty state, not an error crash

## Playwright Automation

Run via Playwright MCP:
```
navigate to /actions
take snapshot
assert main[ref] has children (not empty)
assert no console warning containing "No routes matched"
```
