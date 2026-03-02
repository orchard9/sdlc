# QA Plan: Orchestrator Actions Page

## Scope

Verify the `ActionsPage` UI, sidebar nav entry, SSE integration, recurrence utilities, and API client additions work correctly end-to-end. No new Rust code is introduced by this feature; backend route correctness is covered by `orchestrator-actions-routes` QA.

## Build Verification

- [ ] `cd frontend && npx tsc --noEmit` — zero TypeScript errors
- [ ] `SDLC_NO_NPM=1 cargo build --all` — Rust build succeeds (confirm no regressions)
- [ ] `cargo clippy --all -- -D warnings` — zero new Clippy warnings

## Unit-Level Checks (manual, no test runner required)

### Recurrence utilities

- [ ] `parseRecurrence("10s")` → `10`
- [ ] `parseRecurrence("30m")` → `1800`
- [ ] `parseRecurrence("1h")` → `3600`
- [ ] `parseRecurrence("2d")` → `172800`
- [ ] `parseRecurrence("")` → `null`
- [ ] `parseRecurrence("foo")` → `null`
- [ ] `parseRecurrence("1x")` → `null`
- [ ] `parseRecurrence(" 6h ")` → `21600` (trims whitespace)
- [ ] `formatRecurrence(86400)` → `"1d"`
- [ ] `formatRecurrence(3600)` → `"1h"`
- [ ] `formatRecurrence(60)` → `"1m"`
- [ ] `formatRecurrence(10)` → `"10s"`
- [ ] `formatRecurrence(3601)` → `"3601s"` (no rounding)

## Functional UI Tests (browser, with server running)

### Sidebar navigation

- [ ] "Actions" entry appears in the `setup` group, below "Agents"
- [ ] Clicking "Actions" navigates to `/actions`
- [ ] Active state highlights the "Actions" entry when on `/actions` or any sub-path

### Page loads correctly

- [ ] Page renders without JS errors in the console
- [ ] All three sections are visible: "Scheduled Actions", "Webhook Routes", "Recent Webhook Events"
- [ ] Section headers are visible
- [ ] `[+ Schedule Action]` and `[+ Add Route]` buttons are present

### Scheduled Actions section — empty state

- [ ] When no actions exist, shows: `"No actions scheduled. Use the CLI: sdlc orchestrate add"`

### Scheduled Actions section — with data

- [ ] Actions table renders columns: Label, Tool, Status, Next Run, Recurrence, (edit + delete)
- [ ] Pending status badge is gray
- [ ] Running status badge is blue and pulsing
- [ ] Completed status badge is green
- [ ] Failed status badge is red
- [ ] Recurrence column shows "every 1h" for a 3600-second recurrence
- [ ] Recurrence column shows "—" when recurrence is null
- [ ] Next Run shows relative time for scheduled trigger
- [ ] Next Run shows "webhook-triggered" for webhook trigger type

### Schedule Action modal

- [ ] `[+ Schedule Action]` opens modal
- [ ] Modal has: Label, Tool select, Tool Input textarea, Scheduled At datetime, Recurrence text
- [ ] Tool select is populated from `GET /api/tools`
- [ ] Default Tool Input is `{}`
- [ ] Default Scheduled At is approximately now + 1 minute
- [ ] Recurrence "foo" shows inline validation error below the field
- [ ] Recurrence "1h" clears validation error
- [ ] Submitting valid form calls `POST /api/orchestrator/actions`
- [ ] On success, modal closes and new action appears in table
- [ ] Cancel closes modal without submitting

### Edit Action modal

- [ ] Pencil icon appears in each action row
- [ ] Clicking pencil opens modal pre-populated with current Label and formatted Recurrence
- [ ] Submitting updates the row immediately (optimistic)
- [ ] Server error reverts the row and shows modal error
- [ ] Clearing Recurrence and saving sends `recurrence_secs: null`

### Delete action

- [ ] Trash icon removes action row optimistically on click
- [ ] (If delete fails, row reappears — manual test may require mocking)

### Webhook Routes section — empty state

- [ ] When no routes exist, shows: `"No webhook routes configured."`

### Webhook Routes section — with data

- [ ] Routes table renders columns: Path, Tool, Input Template (truncated), Created, Delete
- [ ] Input template truncated at ~60 chars with ellipsis if longer
- [ ] Created shows relative time

### Add Webhook Route modal

- [ ] `[+ Add Route]` opens modal
- [ ] Modal has: Path, Tool select, Input Template textarea
- [ ] Path without leading `/` shows validation error on submit
- [ ] Tool select populated from `GET /api/tools`
- [ ] Duplicate path shows inline error: `"A route with this path already exists."`
- [ ] Success closes modal and new route appears in table

### Delete webhook route

- [ ] Trash icon removes route row optimistically

### Webhook Events section — empty state

- [ ] When no events exist, shows: `"No webhook events recorded. Events appear here after the first webhook request arrives."`

### Webhook Events section — with data

- [ ] Events table renders columns: Time, Path, Outcome, Action
- [ ] Time shows relative format ("2m ago")
- [ ] Dispatched outcome badge is green
- [ ] NoRouteMatched outcome badge is gray
- [ ] Rejected outcome badge is red
- [ ] Action column shows action label when `action_id` is set
- [ ] Action column shows "—" when `action_id` is null
- [ ] "Load more" link appears when 20 events are shown
- [ ] Clicking "Load more" fetches additional events

### 503 / DB unavailable

- [ ] When the orchestrator DB is unavailable (503 from all endpoints), yellow warning banner appears at top
- [ ] Page body still renders with empty sections (not an error screen)

### SSE real-time updates

- [ ] When the server emits an `action` SSE event, the actions list refetches without manual refresh
- [ ] (Verify by: triggering a tick via CLI, observing status badge change without page reload)

### 5-second polling fallback

- [ ] Even without SSE activity, actions list updates every ~5 seconds
- [ ] (Verify by: checking network tab for periodic `/api/orchestrator/actions` requests)

## Regression Checks

- [ ] All existing sidebar entries remain functional (navigating to other pages works)
- [ ] Existing SSE callbacks (`onUpdate`, `onPonderEvent`, `onRunEvent`, `onInvestigationEvent`, `onAdvisoryEvent`) still fire correctly on their respective events
- [ ] Existing pages (Dashboard, Features, Milestones) load without console errors
- [ ] `Zap` icon does not appear twice or in an incorrect location in the sidebar
