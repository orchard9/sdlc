# Spec: KnowledgePage — three-pane catalog browser in the sdlc UI

## Overview

Add a `KnowledgePage` (`frontend/src/pages/KnowledgePage.tsx`) that lets users browse, search, and inspect the local knowledge base from within the sdlc UI. The page follows the same structural conventions as `GuidelinePage` and `EvolvePage` — a left-pane list, a center or right detail pane, and SSE-driven refresh.

## Routes

| Route | Component |
|---|---|
| `/knowledge` | `KnowledgePage` (no slug — shows empty state in detail pane) |
| `/knowledge/:slug` | `KnowledgePage` (detail pane shows entry for `slug`) |

## Layout

Three-pane layout (left / center / right) — analogous to VS Code's explorer:

```
┌──────────────────────────────────────────────────────────┐
│  Left pane (w-72)   │ Center pane (entry list, w-80) │ Right pane (detail, flex-1) │
│  CatalogTree         │ EntryList                       │ EntryDetail                 │
└──────────────────────────────────────────────────────────┘
```

On mobile:
- Left pane only (entry list view).
- `/knowledge/:slug` hides the left pane and shows the detail pane (same pattern as `GuidelinePage`).

## Left Pane — CatalogTree

- Fetches `GET /api/knowledge/catalog` on mount.
- Renders a collapsible tree of catalog classes and their divisions:
  - Each **class** row: `code` badge + `name`. Clicking expands/collapses divisions.
  - Each **division** row (indented): `code` badge + `name`. Clicking sets the active code filter.
- A **"All"** row at the top clears the code filter.
- A **search bar** (text input) at the top of the pane for full-text search — debounced 300ms, sets `searchQuery` state.
- Active selection is highlighted with `bg-accent text-accent-foreground`.
- When `searchQuery` is non-empty, the code filter is ignored (search overrides catalog filter).
- Empty state: "Knowledge base not initialized. Run `sdlc knowledge librarian init` to seed from your project."

## Center Pane — EntryList

- Fetches `GET /api/knowledge` (optionally with `?code=<prefix>`) whenever the selected catalog code or search query changes.
- When `searchQuery` is set, calls `GET /api/knowledge/search?q=<query>` (future endpoint — fall back to client-side filter of the full list for now, filtering on `title` and `summary`).
- Each entry row shows:
  - **Title** (`font-medium`, truncate)
  - **Code badge** (`font-mono text-xs px-1.5 py-0.5 rounded bg-muted/60`)
  - **Staleness badges** — one badge per flag in `staleness_flags`:
    - `url_404` → red badge "404"
    - `aged_out` → amber badge "aged"
    - `code_ref_gone` → orange badge "stale ref"
  - **`last_updated`** — relative date (`updated_at`), muted text
- Clicking a row navigates to `/knowledge/<slug>`.
- Selected row highlighted with `bg-accent text-accent-foreground`.
- Empty state when no entries match: "No entries found."
- Loading skeleton: three placeholder rows.

## Right Pane — EntryDetail

Shown when a slug is present in the URL. Fetches `GET /api/knowledge/:slug`.

### Header

- Back button (mobile only, `ArrowLeft` icon, navigates to `/knowledge`)
- `title` as `h2`
- Code badge (`font-mono`)
- Status badge (`draft` / `published`) — styled via `StatusBadge` component

### Body (scrollable)

1. **Content** — `content.md` rendered as Markdown (use existing Markdown rendering pattern from other detail pages, or a `<pre>` fallback if no renderer is present).
2. **Source Provenance** — footer section titled "Sources":
   - Each source shows: `type` icon or label, `url` (linked) or `path`, `captured_at` (date).
   - If no sources, omit section.
3. **Related entries** — "Related" section listing `related[]` codes/slugs as clickable links navigating to `/knowledge/<slug>`.
4. **Research More button** — `[Research More]` button that calls `POST /api/knowledge/:slug/research`. Button shows spinner while running; disables during in-flight request. On success/error show a brief toast or inline message.

### SSE refresh

- Listen for `KnowledgeResearchCompleted` and `KnowledgeMaintenanceCompleted` SSE events on the `knowledge` event channel.
- On either event, reload the entry list (re-fetch `GET /api/knowledge`).

## Sidebar

Add a `Library` icon (`lucide-react`) entry to the `plan` group in `Sidebar.tsx`, below `Guidelines`:

```ts
{ path: '/knowledge', label: 'Knowledge', icon: Library, exact: false },
```

## Bottom Tab Bar

Add `/knowledge` to the `roots` array of the `Plan` tab in `BottomTabBar.tsx`:

```ts
roots: ['/feedback', '/ponder', '/investigations', '/evolve', '/guidelines', '/knowledge'],
```

## App Router

Register routes in `App.tsx`:

```tsx
<Route path="/knowledge" element={<KnowledgePage />} />
<Route path="/knowledge/:slug" element={<KnowledgePage />} />
```

## API Client

Add to `frontend/src/api/client.ts`:

```ts
getKnowledgeCatalog: () => request<KnowledgeCatalog>('/api/knowledge/catalog'),
listKnowledge: (code?: string) =>
  request<KnowledgeEntrySummary[]>(`/api/knowledge${code ? `?code=${encodeURIComponent(code)}` : ''}`),
getKnowledgeEntry: (slug: string) => request<KnowledgeEntryDetail>(`/api/knowledge/${slug}`),
researchKnowledge: (slug: string) =>
  request<{ status: string }>(`/api/knowledge/${slug}/research`, { method: 'POST' }),
```

## Types

Add to `frontend/src/lib/types.ts`:

```ts
export interface KnowledgeCatalogDivision {
  code: string
  name: string
  description?: string
}

export interface KnowledgeCatalogClass {
  code: string
  name: string
  description?: string
  divisions: KnowledgeCatalogDivision[]
}

export interface KnowledgeCatalog {
  classes: KnowledgeCatalogClass[]
  updated_at: string
}

export interface KnowledgeSource {
  type: string
  url?: string
  path?: string
  workspace?: string
  captured_at: string
}

export interface KnowledgeEntrySummary {
  slug: string
  title: string
  code: string
  status: string
  summary?: string
  tags: string[]
  created_at: string
  updated_at: string
}

export interface KnowledgeEntryDetail extends KnowledgeEntrySummary {
  sources: KnowledgeSource[]
  related: string[]
  origin: string
  harvested_from?: string
  last_verified_at?: string
  staleness_flags: string[]
  content: string
}

export interface KnowledgeSseEvent {
  type: 'knowledge_research_completed' | 'knowledge_maintenance_completed'
  slug?: string
}
```

## SSE Extension

The `KnowledgePage` uses `useSSE` for generic `onUpdate` callback only (same as `GuidelinePage` — no new SSE event type required for the initial implementation). The `knowledge` SSE channel types (`KnowledgeResearchCompleted`, `KnowledgeMaintenanceCompleted`) are noted in this spec for future extension but the backend does not yet emit them; the page will simply re-fetch on `onUpdate`.

## No PhaseStrip

Knowledge entries have no lifecycle phases (no `draft → specified → ...` flow). The `PhaseStrip` component is not used on this page.

## Out of Scope (Future)

- Creating or editing entries from the UI (write operations beyond "Research More")
- Session log viewer
- Maintenance log viewer
- Full-text search endpoint (`GET /api/knowledge/search`) — fall back to client-side filter in this iteration
