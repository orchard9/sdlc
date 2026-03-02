# QA Results — orchestrator-actions-page

**Date:** 2026-03-02
**Verdict:** PASS

---

## 1. Build Verification

### 1.1 TypeScript (`npx tsc --noEmit`)

**Result: PASS — zero errors**

All new types, components, and imports compile clean. Key type-checked paths:

- `OrchestratorAction`, `OrchestratorWebhookRoute`, `OrchestratorWebhookEvent`, `ActionSseEvent` in `types.ts`
- `parseRecurrence`/`formatRecurrence` in `recurrence.ts`
- 8 new API methods in `client.ts` against their declared return types
- `ActionsPage.tsx` — discriminated union narrowing for `action.trigger`, `action.status`, `event.outcome`
- `SseContext.tsx` — `onActionEvent` callback parameter typed as `ActionSseEvent`

### 1.2 Rust Build

Pre-existing failure in `crates/sdlc-server/src/routes/knowledge.rs` (`E0432: unresolved import 'crate::state::CitedEntry'`) that predates this feature. Confirmed via `git stash` isolation test — error exists on a clean stash of this feature's changes. Not introduced by this feature.

---

## 2. Recurrence Utility Unit Tests

**Result: PASS — 21/21 tests passed**

Executed via `/tmp/test-recurrence.mjs` against the exact logic in `frontend/src/lib/recurrence.ts`.

### parseRecurrence — 11 cases

| Input | Expected | Result |
|-------|----------|--------|
| `"30s"` | 30 | PASS |
| `"5m"` | 300 | PASS |
| `"2h"` | 7200 | PASS |
| `"1d"` | 86400 | PASS |
| `"0m"` | 0 | PASS |
| `""` (empty) | null | PASS |
| `"invalid"` | null | PASS |
| `"1.5h"` (decimal) | null | PASS |
| `"10x"` (bad unit) | null | PASS |
| `"m5"` (reversed) | null | PASS |
| `"  5m  "` (whitespace) | 300 | PASS |

### formatRecurrence — 10 cases

| Input (secs) | Expected | Result |
|-------------|----------|--------|
| 86400 | `"1d"` | PASS |
| 172800 | `"2d"` | PASS |
| 3600 | `"1h"` | PASS |
| 7200 | `"2h"` | PASS |
| 3660 | `"61m"` (not 1h) | PASS |
| 60 | `"1m"` | PASS |
| 300 | `"5m"` | PASS |
| 30 | `"30s"` | PASS |
| 1 | `"1s"` | PASS |
| 90 | `"90s"` (not 1.5m) | PASS |

---

## 3. Static Code Review (QA Plan Acceptance Criteria)

### AC-1: Page renders without crashing when orchestrator DB is unavailable

**PASS** — `Promise.allSettled` pattern used in `fetchAll()` ensures partial failures don't throw. DB unavailability detected from 503/unavailable error message and surfaces the `AlertTriangle` banner. Sections render with empty states.

### AC-2: Scheduled actions table displays all fields correctly

**PASS** — Table columns: Label, Tool (monospace), Status (`ActionStatusBadge`), Next Run (`futureRelativeTime` for scheduled / "webhook-triggered" for webhook), Recurrence (`formatRecurrence` with "every" prefix or "—"), and Actions (edit/delete buttons). Reviewed in `ScheduledActionsSection`.

### AC-3: Schedule Action modal validates recurrence format

**PASS** — `validateRecurrence()` returns false for null result from `parseRecurrence`. Sets `recurrenceError` state and blocks form submission. Error message: "Use format: 10s, 30m, 1h, 2d". Confirmed by unit tests covering all invalid formats.

### AC-4: Delete action is optimistic with revert-on-failure

**PASS** — `handleDelete` in `ScheduledActionsSection`: calls `onActionDeleted(id)` (optimistic state removal) before `await api.deleteAction(id)`. `catch` block calls `onRefresh()` to restore server state. Row renders with `opacity-40` during deletion via `deletingId` state.

### AC-5: Webhook routes table shows path, tool, template, and created

**PASS** — Table shows: Path (monospace), Tool (monospace), Input Template (truncated at 60 chars with ellipsis), Created (`relativeTime`). Delete button per row with same optimistic-delete pattern.

### AC-6: Add Route modal validates path starts with /

**PASS** — `handleSubmit` in `AddRouteModal`: checks `!path.startsWith('/')`, sets `pathError`, returns early. Duplicate path conflicts surfaced from API error message (detects "duplicate", "conflict", "already"). Separate error display location for path vs. general errors.

### AC-7: Webhook events table shows time, path, outcome badge, and dispatched action

**PASS** — `WebhookEventsSection`: Time (relative, with ISO title tooltip), Path (monospace), `OutcomeBadge` (dispatched=green, no_route_matched=muted, rejected=red), Action label looked up via `findActionLabel` from actions list (shows "—" if no match).

### AC-8: Load More button appears when events.length >= limit

**PASS** — Conditional `{events.length >= limit && <button onClick={onLoadMore}>Load more</button>}`. `handleLoadMoreEvents` increments both `eventsLimit.current` (for stable closure) and `eventsLimitState` (for prop), then fetches directly.

### AC-9: SSE subscription triggers action refetch

**PASS** — `useEffect` subscribes via `useSseContext().subscribe({ onActionEvent: () => refetchActions() })`. Returns unsubscribe on cleanup. Supplement: 5-second `setInterval` polling fallback also installed.

### AC-10: Navigation — Actions appears in Sidebar under setup group

**PASS** — `{ path: '/actions', label: 'Actions', icon: Zap, exact: false }` added to `setup` group in `Sidebar.tsx`. Route `<Route path="/actions" element={<ActionsPage />} />` registered in `App.tsx`.

---

## 4. Observations

These are non-blocking notes for future improvement:

- **OBS-1:** `formatRecurrence` in `EditActionModal` pre-fills recurrence from `action.recurrence_secs` correctly; however if the server stores a value like `3661` (non-round seconds), it will display as `"3661s"`, which is technically correct but visually unusual. Acceptable — users set recurrence via the input which only accepts valid patterns.

- **OBS-2:** `WebhookEventsSection` resolves action labels by searching the already-loaded `actions` array. If an event's `action_id` refers to an action that was deleted, the label shows "—". This is correct behavior, not a bug.

- **OBS-3:** The `AlertTriangle` DB unavailable banner only fires when the actions endpoint returns 503/unavailable. Routes and events endpoints do not independently trigger the banner — they fail silently, leaving their sections empty. Acceptable for v1; can be improved with per-section error indicators in a follow-up.

---

## 5. Summary

| Category | Result |
|----------|--------|
| TypeScript compilation | PASS |
| Recurrence unit tests (21/21) | PASS |
| All 10 acceptance criteria | PASS |
| Pre-existing Rust build failure | PRE-EXISTING (not caused by this feature) |

**Overall: PASS — ready for merge.**
