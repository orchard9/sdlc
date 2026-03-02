# Tasks: Orchestrator Actions Page

## T1 — Add TypeScript types to types.ts

Add `OrchestratorAction`, `OrchestratorWebhookRoute`, `OrchestratorWebhookEvent`, and `ActionSseEvent` to `frontend/src/lib/types.ts`.

**Acceptance:** Types compile without errors and are importable from other modules.

---

## T2 — Create recurrence utility module

Create `frontend/src/lib/recurrence.ts` with `parseRecurrence(s: string): number | null` and `formatRecurrence(secs: number): string`.

**Acceptance:** `parseRecurrence("1h")` returns `3600`, `parseRecurrence("foo")` returns `null`, `formatRecurrence(3600)` returns `"1h"`, `formatRecurrence(10)` returns `"10s"`.

---

## T3 — Add orchestrator API methods to client.ts

Append 7 new methods to the `api` object in `frontend/src/api/client.ts`:
- `listActions()`
- `createAction(body)`
- `updateAction(id, patch)`
- `deleteAction(id)`
- `listWebhookRoutes()`
- `createWebhookRoute(body)`
- `deleteWebhookRoute(id)`
- `listWebhookEvents(limit?)`

**Acceptance:** All methods match the API shapes defined in the design doc. TypeScript compiles.

---

## T4 — Add ActionSseEvent to SseContext

Extend `frontend/src/contexts/SseContext.tsx`:
1. Add `onActionEvent?: (event: ActionSseEvent) => void` to `SseCallbacks`
2. Add `else if (type === 'action')` branch in `dispatch()` that parses and dispatches the event

**Acceptance:** SSE `action` events are dispatched to any subscribed `onActionEvent` callback. Existing callbacks (`onUpdate`, `onPonderEvent`, etc.) are unaffected.

---

## T5 — Add Actions entry to sidebar nav

In `frontend/src/components/layout/Sidebar.tsx`, add to the `setup` group after Agents:

```typescript
{ path: '/actions', label: 'Actions', icon: Zap, exact: false }
```

**Acceptance:** "Actions" appears in the sidebar under the `setup` group. Clicking it navigates to `/actions`. Active state highlights correctly.

---

## T6 — Register /actions route in App.tsx

In `frontend/src/App.tsx`:
1. Add `import { ActionsPage } from '@/pages/ActionsPage'`
2. Add `<Route path="/actions" element={<ActionsPage />} />` after the Agents route

**Acceptance:** Navigating to `/actions` renders the `ActionsPage` component without a 404 or blank screen.

---

## T7 — Implement ActionsPage with Scheduled Actions section

Create `frontend/src/pages/ActionsPage.tsx` with:
- `useActionsData()` hook using `Promise.allSettled` for all three fetches
- `ScheduledActionsSection` with table (Label, Tool, Status badge, Next Run, Recurrence, Edit/Delete actions)
- `ActionStatusBadge` with correct color per status
- Empty state: `"No actions scheduled. Use the CLI: sdlc orchestrate add"`
- 503 warning banner at page top when DB is unavailable

**Acceptance:** Page renders with correct table structure. Status badges show correct colors. Empty state displays when no actions exist. 503 shows warning banner, not a blank error.

---

## T8 — Implement Schedule Action modal

Add `ScheduleActionModal` inside `ActionsPage.tsx`:
- Fields: Label, Tool (select from `api.listTools()`), Tool Input (JSON textarea, default `{}`), Scheduled At (datetime-local, default now+1min), Recurrence (text input)
- Recurrence validation: inline error if input does not match `/^(\d+)(s|m|h|d)$/` and is not empty
- Submit: `api.createAction(...)` → close modal → refresh actions list

**Acceptance:** Modal opens on `[+ Schedule Action]`, all fields are present, invalid recurrence shows inline error, valid submission creates action and closes modal.

---

## T9 — Implement Edit Action modal

Add `EditActionModal` inside `ActionsPage.tsx`:
- Pencil icon per row in the actions table
- Pre-populates Label and Recurrence from selected action (using `formatRecurrence` for display)
- Submit: `api.updateAction(id, { label, recurrence_secs })` — optimistic update then revert on failure
- `recurrence_secs: null` when Recurrence field is empty (clears recurrence)

**Acceptance:** Pencil icon opens modal pre-populated. Submitting updates the row immediately. Server error reverts the change and shows inline modal error.

---

## T10 — Implement Webhook Routes section

Add `WebhookRoutesSection` inside `ActionsPage.tsx`:
- Table: Path, Tool, Input Template (truncated at 60 chars), Created (relative time), Delete
- Delete: `api.deleteWebhookRoute(id)` — optimistic removal, revert on failure
- Empty state: `"No webhook routes configured."`
- `[+ Add Route]` opens `AddRouteModal`

**Acceptance:** Routes table renders correctly. Delete removes row optimistically. Empty state displays correctly.

---

## T11 — Implement Add Webhook Route modal

Add `AddRouteModal` inside `ActionsPage.tsx`:
- Fields: Path (must start with `/`), Tool (select from `api.listTools()`), Input Template (textarea)
- 409 error shows inline: `"A route with this path already exists."`
- On success: close modal, refresh routes

**Acceptance:** Modal opens on `[+ Add Route]`. All validations work. 409 shows inline error. Successful submission closes modal and shows new route.

---

## T12 — Implement Webhook Events section

Add `WebhookEventsSection` inside `ActionsPage.tsx`:
- Table: Time (relative, tooltip with absolute), Path, Outcome badge, Action (link to action row or "—")
- `OutcomeBadge` with color per outcome type
- Default 20 events; "Load more" link visible when exactly 20 events returned
- Empty state: `"No webhook events recorded. Events appear here after the first webhook request arrives."`

**Acceptance:** Events table renders correctly. Outcome badges have correct colors. "Load more" fetches next 20. Empty state displays when no events.

---

## T13 — Wire SSE and polling to ActionsPage

In `ActionsPage`:
1. Subscribe to `onActionEvent` via `useSseContext()` — calls `refetchActions()` on each event
2. Add `useEffect` with `setInterval(refetchActions, 5000)` for fallback polling

**Acceptance:** When a `action_state_changed` SSE event arrives, the actions list refetches. Without SSE, the list still polls every 5 seconds.

---

## T14 — Verify TypeScript build passes

Run `cd frontend && npx tsc --noEmit` to confirm no type errors across all changed files.

**Acceptance:** TypeScript compiler reports zero errors.

---

## T15 — Verify Rust build passes

Run `SDLC_NO_NPM=1 cargo build --all` and `cargo clippy --all -- -D warnings` to confirm no regressions from the SseMessage variant addition.

**Note:** `SseMessage::ActionStateChanged` is added to `state.rs` as part of the `orchestrator-actions-routes` feature. This task confirms the server compiles with the new variant in place (or confirms it compiles without it if that feature ships separately).

**Acceptance:** `cargo build --all` completes without errors or new warnings.
