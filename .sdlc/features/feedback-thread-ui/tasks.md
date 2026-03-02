# Tasks: FeedbackThread UI

## T1 — Add TypeScript types for FeedbackThread

Add `ThreadSummary`, `ThreadDetail`, `ThreadComment`, and `ThreadStatus` to `frontend/src/lib/types.ts`.

**Files:** `frontend/src/lib/types.ts`

---

## T2 — Add thread API methods to api/client.ts

Add `listThreads`, `getThread`, `createThread`, `addThreadComment`, and `resolveThreadComment` to the `api` object in `frontend/src/api/client.ts`.

**Files:** `frontend/src/api/client.ts`

---

## T3 — Create ThreadsPage with routing

Create `frontend/src/pages/ThreadsPage.tsx` with:
- Two-pane layout (list + detail)
- `useParams` for selected slug
- Load thread list on mount
- Load thread detail when slug changes
- Empty states for both panes

Register routes in `frontend/src/App.tsx`:
```tsx
<Route path="/threads" element={<ThreadsPage />} />
<Route path="/threads/:slug" element={<ThreadsPage />} />
```

**Files:** `frontend/src/pages/ThreadsPage.tsx`, `frontend/src/App.tsx`

---

## T4 — Create ThreadListPane component

Create `frontend/src/components/threads/ThreadListPane.tsx` with:
- "Threads" heading
- "New thread" button (calls `onNewThread` prop)
- List of `ThreadListItem` rows (title, status badge, comment count, author)
- Selected state highlighting
- Empty state when no threads

**Files:** `frontend/src/components/threads/ThreadListPane.tsx`

---

## T5 — Create ThreadDetailPane component

Create `frontend/src/components/threads/ThreadDetailPane.tsx` with:
- Thread header (title, status badge, meta, Synthesize/Promote stub buttons)
- Scrollable body area
- `CoreElement` card
- Section divider
- `CommentList` (maps `ThreadComment[]` to `CommentCard`)
- `ComposeArea` sticky footer

**Files:** `frontend/src/components/threads/ThreadDetailPane.tsx`

---

## T6 — Create CoreElement component

Create `frontend/src/components/threads/CoreElement.tsx` with:
- "Core element" label + version chip (`v{body_version}`)
- Body content (plain text pre-wrap, or muted placeholder if null)
- Version history strip (current version chip only in V1)

**Files:** `frontend/src/components/threads/CoreElement.tsx`

---

## T7 — Create CommentCard component

Create `frontend/src/components/threads/CommentCard.tsx` with:
- Avatar (initial letter, green for humans, blue-purple for `agent:*`)
- Author, timestamp
- `incorporated` → dimmed styling + dashed border + "absorbed" badge
- Body text

**Files:** `frontend/src/components/threads/CommentCard.tsx`

---

## T8 — Create NewThreadModal component

Create `frontend/src/components/threads/NewThreadModal.tsx` with:
- Title input (required, autofocus)
- Core element textarea (optional)
- Cancel / Create thread buttons
- Calls `onSubmit({ title, body? })` on submit
- Closes on Escape and Cancel

**Files:** `frontend/src/components/threads/NewThreadModal.tsx`

---

## T9 — Wire up compose area (send comment)

In `ThreadDetailPane` (or a sub-component `ComposeArea`):
- Controlled textarea with `composeDraft` state
- `Cmd+Enter` keyboard shortcut
- Calls `api.addThreadComment(slug, { author: 'jordan', body: draft })`
- Optimistic append to comment list
- Clears draft on success
- Shows inline error on failure

**Files:** `frontend/src/components/threads/ThreadDetailPane.tsx`

---

## T10 — Update Sidebar nav (replace Feedback with Threads)

In `frontend/src/components/layout/Sidebar.tsx`:
- Replace `{ path: '/feedback', label: 'Feedback', icon: MessageSquarePlus, exact: false }` with `{ path: '/threads', label: 'Threads', icon: MessagesSquare, exact: false }`
- Import `MessagesSquare` from `lucide-react`

**Files:** `frontend/src/components/layout/Sidebar.tsx`

---

## T11 — Update AppShell for Threads route

In `frontend/src/components/layout/AppShell.tsx`:
- Add `'/threads': 'Threads'` to `PATH_LABELS`
- Add `'/threads/'` to `DETAIL_BASES` for mobile back-button

**Files:** `frontend/src/components/layout/AppShell.tsx`

---

## T12 — Import ThreadsPage in App.tsx

Import `ThreadsPage` into `frontend/src/App.tsx` and add both routes.

**Files:** `frontend/src/App.tsx`

---

## T13 — Verify build compiles without TypeScript errors

Run `cd frontend && npm run build` (or `npm run typecheck` if available) to confirm the new components type-check cleanly. Fix any errors found.

**Files:** (various, depending on errors)
