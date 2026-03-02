# Design: FeedbackThread UI

## Architecture Overview

This is a pure frontend change. No new Rust code; no new server routes in this feature. All data flows through the REST endpoints delivered by `feedback-thread-core`.

```
Browser                      sdlc-server
──────                       ──────────
ThreadsPage
  ├── ThreadListPane  ──GET /api/threads──────────────────────────►
  │                  ◄── ThreadSummary[] ─────────────────────────
  └── ThreadDetailPane
        ├── CoreElement
        ├── CommentList ──GET /api/threads/:slug/comments──────────►
        │              ◄── ThreadComment[] ─────────────────────────
        └── ComposeArea ──POST /api/threads/:slug/comments─────────►
                        ◄── ThreadComment ───────────────────────────
NewThreadModal ─────────POST /api/threads──────────────────────────►
               ◄────────ThreadSummary ──────────────────────────────
```

## Component Tree

```
App.tsx
└── AppShell
    ├── Sidebar (modified — Threads nav item)
    └── main
        └── ThreadsPage
            ├── ThreadListPane
            │   ├── list header + "New thread" button
            │   └── ThreadListItem × N
            ├── ThreadDetailPane            (selected thread)
            │   ├── ThreadHeader
            │   │   ├── title + StatusBadge
            │   │   ├── meta (author · date · comment count)
            │   │   └── action buttons (Synthesize stub, Promote stub)
            │   ├── thread-body-area (scrollable)
            │   │   ├── CoreElement card
            │   │   │   ├── core-element-header (label + version)
            │   │   │   ├── body content (markdown or placeholder)
            │   │   │   └── VersionStrip
            │   │   ├── section divider (N comments)
            │   │   └── CommentList
            │   │       └── CommentCard × N
            │   └── ComposeArea (sticky footer)
            └── NewThreadModal (conditional)
```

## File Structure

```
frontend/src/
├── pages/
│   └── ThreadsPage.tsx        (new)
├── components/
│   └── threads/               (new directory)
│       ├── ThreadListPane.tsx
│       ├── ThreadDetailPane.tsx
│       ├── CoreElement.tsx
│       ├── CommentCard.tsx
│       └── NewThreadModal.tsx
├── api/
│   └── client.ts              (modified — thread API methods)
└── lib/
    └── types.ts               (modified — ThreadSummary, ThreadDetail, ThreadComment)
```

Components under `frontend/src/components/threads/` follow the same pattern as `frontend/src/components/investigation/` and `frontend/src/components/ponder/`.

## Layout Behaviour

### Desktop (≥ md breakpoint)

Fixed two-pane layout:
- Left pane: `w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden`
- Right pane: `flex-1 flex flex-col overflow-hidden`

This is identical to the approach in PonderPage and InvestigationPage where a list panel and detail panel sit side by side.

### Mobile (< md)

- `/threads` renders only the thread list (full width)
- `/threads/:slug` renders only the thread detail (full width)
- Mobile header shows a back button chevron (AppShell's `isDetailView` logic) — achieved by adding `/threads/` to `DETAIL_BASES` in AppShell.tsx

## State Management

All state is local to `ThreadsPage` (no global store needed). The page uses React's `useState` and `useEffect` with the existing `api` client.

```ts
// ThreadsPage internal state
const [threads, setThreads] = useState<ThreadSummary[]>([])
const [loadingList, setLoadingList] = useState(true)
const [detail, setDetail] = useState<ThreadDetail | null>(null)
const [loadingDetail, setLoadingDetail] = useState(false)
const [composeDraft, setComposeDraft] = useState('')
const [composing, setComposing] = useState(false)
const [createOpen, setCreateOpen] = useState(false)
```

URL is the source of truth for selected thread — `useParams()` drives `selectedSlug`.

## Data Flows

### Loading threads list
```
mount → api.listThreads() → setThreads → render ThreadListPane
```

### Selecting a thread
```
URL change (/:slug) → useEffect on slug → api.getThread(slug) → setDetail → render ThreadDetailPane
```

### Sending a comment
```
Submit → api.addThreadComment(slug, { author, body })
       → optimistic: append to detail.comments immediately
       → on success: nothing (already done)
       → on error: remove optimistic comment, show error
       → clear composeDraft
```

### Creating a thread
```
Modal submit → api.createThread({ title, body })
             → navigate(`/threads/${newThread.slug}`)
             → setThreads(prev => [newThread, ...prev])
             → close modal
```

## Status Badge Colour Mapping

| Status      | Background                         | Text                   |
|-------------|------------------------------------|------------------------|
| open        | `bg-green-950/40` or `bg-primary/10`| `text-primary`         |
| synthesized | `bg-indigo-950/40`                 | `text-indigo-400`      |
| promoted    | `bg-muted`                         | `text-muted-foreground`|

Match the HTML prototype colours as closely as possible within the existing Tailwind token set.

## Author Avatar Logic

```ts
function isAgent(author: string): boolean {
  return author.startsWith('agent:')
}

function avatarInitial(author: string): string {
  const name = author.startsWith('agent:') ? author.slice(6) : author
  return name.charAt(0).toUpperCase()
}
```