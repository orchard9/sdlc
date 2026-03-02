# Spec: FeedbackThread UI — thread list, detail view, and sidebar nav

## Summary

Replace the current `FeedbackPage` (global anonymous note queue) with a two-pane `ThreadsPage` that surfaces the FeedbackThread primitive delivered by `feedback-thread-core`. Users see a list of threads on the left, select one to read and comment on the right, and can create new threads via a modal. A "Threads" nav item is added to the sidebar in the `plan` group, replacing the existing "Feedback" item.

This is the UI layer only. It consumes the REST API defined in `feedback-thread-core` — no data model or Rust changes in this feature.

---

## Background

The current `FeedbackPage` uses a global capture queue (`feedback.yaml`) that is anonymous, context-free, and destructively consumed on submit. The design prototype at `.sdlc/roadmap/i-need-a-better-feedback-system-that-let/feedback-threads-ui.html` defines the target UX: a master-detail layout with a thread list pane and a scrollable thread detail pane containing a core element (body), a comment list, and a compose box.

The ponder artifacts establish these decisions:
- "Threads" is a standalone nav item in the `plan` group (not embedded in Ponder)
- Threads can be created with title-only (body is optional)
- Comments have `author` attribution (human or `agent:*` prefix)
- `incorporated: bool` on each comment indicates synthesis absorption (V2; rendered as visual dimming in V1)
- Synthesis and Promote buttons are present in the UI as stubs (disabled/placeholder in V1; wire up in V2)

---

## REST API Contract (provided by feedback-thread-core)

The UI depends on these endpoints from the sibling feature:

```
GET    /api/threads                          → ThreadSummary[]
POST   /api/threads                          → ThreadSummary         body: { title, body? }
GET    /api/threads/:slug                    → ThreadDetail
POST   /api/threads/:slug/comments           → Comment               body: { author?, body }
GET    /api/threads/:slug/comments           → Comment[]
PATCH  /api/threads/:slug/comments/:id       → Comment               body: { resolved? }
```

TypeScript shapes (added to `frontend/src/lib/types.ts`):

```ts
export type ThreadStatus = 'open' | 'synthesized' | 'promoted'

export interface ThreadSummary {
  slug: string
  title: string
  author: string
  status: ThreadStatus
  comment_count: number
  created_at: string
  updated_at: string
  promoted_to: string | null
}

export interface ThreadComment {
  id: string
  author: string
  body: string
  incorporated: boolean
  created_at: string
}

export interface ThreadDetail extends ThreadSummary {
  body: string | null          // core element body (null if title-only)
  body_version: number
  comments: ThreadComment[]
}
```

---

## Pages and Routes

### New: ThreadsPage (`/threads` and `/threads/:slug`)

Single file: `frontend/src/pages/ThreadsPage.tsx`

Two-pane layout (mirrors the HTML prototype):
- **Left pane** (280px fixed): thread list + "New thread" button at top
- **Right pane** (flex-1): thread detail with header, scrollable body area, and compose footer

Route behaviour:
- `/threads` — list loaded, no thread selected; right pane shows empty state
- `/threads/:slug` — selected thread auto-loaded and displayed in right pane

The two-pane layout is always rendered on desktop. On mobile, use the same pattern as Ponder/Investigation: the list renders at `/threads` and a detail at `/threads/:slug` replaces it (full-screen).

### Retiring: FeedbackPage

`/feedback` stays in the router but its nav entry in `Sidebar.tsx` is replaced. The existing page is not deleted — it can stay as a fallback route. The sidebar "Feedback" item (`MessageSquarePlus` icon) is replaced by a "Threads" item (`MessageSquareText` or `MessagesSquare` icon from lucide-react).

---

## Component Breakdown

### ThreadsPage

State:
- `threads: ThreadSummary[]` — loaded on mount from `GET /api/threads`
- `selectedSlug: string | null` — driven by URL param
- `detail: ThreadDetail | null` — loaded when `selectedSlug` changes
- `composeDraft: string` — controlled textarea value
- `composing: boolean` — send in progress
- `createOpen: boolean` — new thread modal open

Behaviour:
- On mount: `api.listThreads()` → set `threads`
- On `selectedSlug` change: `api.getThread(slug)` → set `detail`
- Sending a comment: `api.addThreadComment(slug, { author: 'jordan', body: draft })` → append to local `detail.comments`, clear draft
- Creating a thread: POST to `api.createThread({ title, body })` → close modal, navigate to `/threads/:newSlug`, reload list

No auto-refresh needed for V1 (load on mount + mutation). SSE can wire up in a follow-on task once the thread SSE events are defined.

### ThreadListPane

Renders the sorted list of `ThreadSummary` items. Each row shows:
- Thread title (truncated with ellipsis)
- Status badge: `open` (green), `synthesized` (blue-purple), `promoted` (grey with "→ ponder")
- Comment count
- Author

Selected item has `bg-accent` background and a subtle border (matches the HTML prototype's `.selected` style).

### ThreadDetailPane

Three sections:
1. **Header** — title, status badge, meta (author, date, comment count), action buttons (Synthesize stub, Promote stub)
2. **Scrollable body area** — CoreElement card + section divider + CommentList
3. **ComposeArea** — sticky footer with author attribution, textarea, Send button

#### CoreElement

Renders the thread body as a styled card with:
- "Core element" label + version indicator (`v{body_version}`)
- Body content (Markdown-rendered via `react-markdown` if already in the project; plain `<pre>` otherwise)
- Version history strip (just the current version chip in V1; clicking is a no-op)

If `body` is null (title-only thread), render a muted "No core element yet" placeholder.

#### CommentList

Maps `ThreadDetail.comments` to `CommentCard` items.

`CommentCard`:
- Avatar: first letter of author, green background for humans, blue-purple for `agent:*` authors
- Author name + timestamp
- `incorporated` flag → `opacity-50` + dashed border + "absorbed" badge
- Body text (plain, preserve newlines)

#### ComposeArea

- Author attribution row (hardcoded "jordan" in V1; can be made configurable later)
- Textarea with `⌘↵` hint and Send button
- `Cmd+Enter` keyboard shortcut submits

### NewThreadModal

Fields:
- **Title** (required) — `<input>` with autofocus
- **Core element** (optional) — `<textarea>` with hint "Becomes the living summary. Leave blank to fill in later."

Footer: Cancel (ghost) + Create thread (primary)

---

## Navigation Changes

### Sidebar.tsx

Replace:
```ts
{ path: '/feedback', label: 'Feedback', icon: MessageSquarePlus, exact: false },
```
With:
```ts
{ path: '/threads', label: 'Threads', icon: MessagesSquare, exact: false },
```

Import `MessagesSquare` from `lucide-react` (or `MessageSquareText` — verify availability).

### AppShell.tsx

Add to `PATH_LABELS`:
```ts
'/threads': 'Threads',
```

Also add `/threads/` to `DETAIL_BASES` so the mobile back-button pattern works.

### App.tsx

Add routes:
```tsx
<Route path="/threads" element={<ThreadsPage />} />
<Route path="/threads/:slug" element={<ThreadsPage />} />
```

---

## API Client Additions (client.ts)

```ts
listThreads: () =>
  request<import('@/lib/types').ThreadSummary[]>('/api/threads'),
getThread: (slug: string) =>
  request<import('@/lib/types').ThreadDetail>(`/api/threads/${encodeURIComponent(slug)}`),
createThread: (body: { title: string; body?: string }) =>
  request<import('@/lib/types').ThreadSummary>('/api/threads', {
    method: 'POST', body: JSON.stringify(body),
  }),
addThreadComment: (slug: string, data: { author?: string; body: string }) =>
  request<import('@/lib/types').ThreadComment>(`/api/threads/${encodeURIComponent(slug)}/comments`, {
    method: 'POST', body: JSON.stringify(data),
  }),
resolveThreadComment: (slug: string, commentId: string) =>
  request<import('@/lib/types').ThreadComment>(
    `/api/threads/${encodeURIComponent(slug)}/comments/${encodeURIComponent(commentId)}`,
    { method: 'PATCH', body: JSON.stringify({ resolved: true }) }
  ),
```

---

## Out of Scope (V1)

- Synthesis agent run (Synthesize button is a UI stub, disabled)
- Promote to Ponder (button present, disabled with tooltip "coming soon")
- SSE live updates for new comments
- Editable author name (hardcoded "jordan" in V1)
- Comment resolve action (UI renders incorporated state from API; no resolve button in V1)
- Thread search / filter
- Thread status transitions (open → synthesized → promoted managed by backend events in V2)

---

## Acceptance Criteria

1. `/threads` renders the two-pane layout: left pane with thread list, right pane with empty-state prompt
2. Selecting a thread in the list navigates to `/threads/:slug` and loads the detail view
3. Thread detail shows title, status badge, author, created_at, comment count, core element (or placeholder), and comment list
4. Incorporated comments render with dimmed styling
5. Agent-authored comments (`author` starts with `agent:`) show blue-purple avatar; human comments show green avatar
6. The compose box sends a comment via `POST /api/threads/:slug/comments` and appends it to the list without a page reload
7. "New thread" button opens the create-thread modal; submitting creates the thread and navigates to it
8. Sidebar shows "Threads" in the `plan` group and the item is active when on `/threads*`
9. `Cmd+Enter` in the compose textarea submits the comment
10. Empty thread list shows a friendly empty state with a "New thread" prompt
