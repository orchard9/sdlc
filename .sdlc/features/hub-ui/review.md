# Code Review: Hub UI

## Summary

Implementation adds a standalone hub page (`HubPage.tsx`) rendered at `/` when the server is in hub mode. Hub mode is detected once on app load via a probe to `GET /api/hub/projects`. Normal mode is completely unaffected — all existing routes and the `AppShell` layout are unchanged.

## Files Changed

| File | Change |
|---|---|
| `frontend/src/lib/types.ts` | Added `HubProjectEntry`, `HubProjectStatus`, `HubSseEvent` types |
| `frontend/src/api/client.ts` | Added `getHubProjects()` method |
| `frontend/src/hooks/useHubSSE.ts` | New hook — dedicated SSE connection for `/api/hub/events` |
| `frontend/src/pages/HubPage.tsx` | New page — filter, cards, empty state |
| `frontend/src/App.tsx` | Added hub mode detection, conditional render of `HubPage` vs normal app |

## Review Findings

### Architecture

- Hub mode detection uses a simple `fetch` probe in `useHubMode()`, not the `api.getHubProjects()` client method. This is intentional — `api.request()` throws on non-2xx, and we need to distinguish 200 from 503 without an exception. The raw `fetch` is cleaner here.
- `useHubSSE` does not depend on `SseContext` (correct — different endpoint, different lifecycle).
- The `BrowserRouter` and `SseProvider` are not mounted in hub mode. This is intentional since hub mode has no client-side routing or `/api/events` needs.

### Correctness

- Upsert-by-URL logic in `onProjectUpdated` is correct: finds by URL, replaces in place, or prepends if new.
- `onProjectRemoved` correctly filters by URL.
- `statusForAge` mirrors server-side thresholds exactly: <30s online, 30–90s stale, ≥90s offline.
- 15-second recompute interval keeps status dots fresh between SSE events.

### Code Quality

- No `unwrap()` or unsafe patterns (this is frontend TypeScript, not Rust library code, so the Rust convention is not directly applicable, but no dangerous `.!` non-null assertions are used either).
- Components are small and focused: `StatusDot`, `AgentBadge`, `ProjectCard`, `EmptyState`, `HubPage` are each single-responsibility.
- Filter is purely derived from `projects` + `filter` state — no separate derived state variable needed.
- `useCallback` used correctly for SSE handlers to avoid unnecessary re-subscriptions.

### Build Verification

- `npm run build` passes with no TypeScript errors.
- `SDLC_NO_NPM=1 cargo test --all` passes — no Rust changes.
- `cargo clippy --all -- -D warnings` passes — only pre-existing sqlx deprecation warning (unrelated).

### No Issues Found

All acceptance criteria from the spec are met:
1. Hub mode detection based on `/api/hub/projects` response code — verified.
2. Normal mode fallback on 503/error — verified.
3. Filter input filters client-side, case-insensitive on name + URL — implemented.
4. Count text reflects filter state ("N of M" vs "N projects") — implemented.
5. Status dots green/yellow/grey per status — implemented.
6. Agent badge visible only when `agent_running === true` — implemented.
7. Card click opens URL in new tab — `window.open(url, '_blank')` — implemented.
8. SSE updates cards without reload — implemented via `useHubSSE`.
9. Empty state with `~/.sdlc/hub.yaml` hint — implemented.
10. No sidebar/nav in hub mode — `HubPage` renders standalone without `AppShell`.
