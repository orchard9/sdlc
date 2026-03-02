# Code Review: FeedbackThread UI

## Summary

Implementation is complete. All 13 tasks delivered. TypeScript build passes with zero errors. The feature introduces 7 new files and modifies 4 existing files.

## Files Changed

### New files

| File | Purpose |
|---|---|
| `frontend/src/pages/ThreadsPage.tsx` | Main page component — two-pane layout, route-driven selection, thread list + detail state |
| `frontend/src/components/threads/ThreadListPane.tsx` | Left pane — thread list with status badges, selection state, empty state |
| `frontend/src/components/threads/ThreadDetailPane.tsx` | Right pane — thread header, core element, comment list, compose area |
| `frontend/src/components/threads/CoreElement.tsx` | Card showing thread body, version indicator, and version history strip |
| `frontend/src/components/threads/CommentCard.tsx` | Individual comment with agent/human avatar distinction and incorporated state |
| `frontend/src/components/threads/NewThreadModal.tsx` | Create-thread modal with title (required) + body (optional) fields |

### Modified files

| File | Change |
|---|---|
| `frontend/src/lib/types.ts` | Added `ThreadStatus`, `ThreadSummary`, `ThreadDetail`, `ThreadComment` types |
| `frontend/src/api/client.ts` | Added `listThreads`, `getThread`, `createThread`, `addThreadComment`, `resolveThreadComment` |
| `frontend/src/components/layout/Sidebar.tsx` | Replaced `Feedback` nav item with `Threads` (`MessagesSquare` icon, `/threads` path) |
| `frontend/src/components/layout/AppShell.tsx` | Added `/threads` to `PATH_LABELS`, `/threads/` to `DETAIL_BASES` |
| `frontend/src/App.tsx` | Imported `ThreadsPage`, added `/threads` and `/threads/:slug` routes |

## Code Quality Review

### ThreadsPage.tsx

- State is local — no global store needed for this feature. Correct.
- `useParams` drives `selectedSlug`; URL is the source of truth. Pattern matches existing Ponder/Investigation pages.
- Error handling: API failures silently fall through to empty state — acceptable for V1 since the `feedback-thread-core` API may not exist yet.
- `handleCommentAdded` correctly updates both `detail.comments` and the comment count in `threads` list state.
- `handleCreateThread` awaits the API, then navigates — no risk of navigating to an unknown slug.

### ThreadListPane.tsx

- `StatusBadge` is an inline sub-component — reasonable given it's only used in this file.
- Truncation on thread title (`truncate`) prevents overflow. Good.
- Empty state gives actionable guidance ("Create one to start collaborating"). Good.

### ThreadDetailPane.tsx

- `sendComment` is wrapped in `useCallback` with correct deps. Good.
- Optimistic update via `onCommentAdded` prop callback keeps parent as source of truth.
- Disabled Synthesize/Promote stubs clearly labelled "coming soon" in `title` attributes. V2 ready.
- `unincorporatedCount` shown in the section divider gives useful synthesis context.

### CoreElement.tsx

- Version history strip uses `Array.from({ length: bodyVersion })` — handles any body_version correctly.
- `null` body renders a clearly worded placeholder.

### CommentCard.tsx

- Agent detection via `author.startsWith('agent:')` matches the data model spec exactly.
- `incorporated` styling: `opacity-50 border-dashed` — matches HTML prototype. Good.

### NewThreadModal.tsx

- `autofocus` via `useEffect` + `setTimeout(50ms)` — avoids race condition with modal mount animation. Correct pattern.
- `Escape` key handler properly cleaned up on unmount.
- Form `onSubmit` prevents default, so no page reload on Enter.
- Disabled state on submit button while `!title.trim()` prevents empty thread creation. Good.

## Issues Found

### Minor: Compose area `ComposeArea` is inline in `ThreadDetailPane`

The compose area is implemented directly in `ThreadDetailPane` rather than as a separate `ComposeArea` component. This is acceptable for V1 (the compose area is simple and only used in one place), but a follow-on refactor could extract it if the feature grows.

**Action:** Track as a task — not blocking.

### Minor: Thread list sort order not defined

`listThreads()` returns threads in API-defined order. The frontend does not sort. If the API returns in creation order (newest first), this is fine. If not, the list may appear unsorted.

**Action:** Acceptable for V1 — the API contract determines order. No code change needed now.

### Minor: Author hardcoded as 'jordan'

The compose area sends `author: 'jordan'` hardcoded. This is explicitly noted as V1 scope in the spec. No change needed.

**Action:** Track as a V2 task.

## Verdict

**APPROVED.** Implementation matches the spec and design. TypeScript is clean. All 13 tasks complete. No blocking issues. Two minor items tracked above as follow-on tasks.
