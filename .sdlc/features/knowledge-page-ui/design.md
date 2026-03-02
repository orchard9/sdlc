# Design: KnowledgePage — three-pane catalog browser

## Layout Wireframes

### Desktop — No selection (catalog tree open, no entry selected)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ SDLC                                                                          │
│ Feature Lifecycle                                                             │
├─────────────┬─────────────────────────────────────────────────────────────────┤
│ work        │                                                                 │
│  Dashboard  │  [Left Pane w-72]  [Center Pane w-80]  [Right Pane flex-1]    │
│  Milestones │  ┌──────────────┐  ┌──────────────────┐  ┌──────────────────┐ │
│  Features   │  │ 🔍 Search... │  │  100 · Philosophy │  │                  │ │
│             │  ├──────────────┤  │  ──────────────── │  │  Select an entry │ │
│ plan        │  │ All          │  │  [aged] · 12d ago │  │  to view details │ │
│  Feedback   │  │ ▼ 100 · Phil │  │                  │  │                  │ │
│  Ponder     │  │   100.20 Eth │  │  200 · Science   │  │                  │ │
│  Root Cause │  │   100.30 Log │  │  ──────────────── │  │                  │ │
│  Evolve     │  │ ▶ 200 · Sci  │  │  Auth JWT Patt.. │  │                  │ │
│  Guidelines │  │ ▶ 300 · Tech │  │  [404] · 5d ago  │  │                  │ │
│  Knowledge  │  └──────────────┘  │                  │  │                  │ │
│             │                    │  300 · Tech      │  │                  │ │
│ setup       │                    │  ──────────────── │  │                  │ │
│ project     │                    │  ...             │  │                  │ │
└─────────────┴────────────────────┴──────────────────┴──────────────────────┘
```

### Desktop — Entry selected

```
┌─────────────┬────────────────────┬──────────────────┬──────────────────────┐
│             │  ┌──────────────┐  │                  │  ← Back (mobile)     │
│             │  │ 🔍 auth jwt  │  │  [selected row]  │  JWT Authentication  │
│             │  ├──────────────┤  │  ▶ Auth JWT Pa.. │  Pattern             │
│             │  │ All          │  │    [404]         │  [200] [published]   │
│             │  │ ▼ 200 · Sci  │  │                  │  ─────────────────── │
│             │  │   200.10 CS  │  │                  │  # JWT Auth Pattern  │
│             │  │   200.20 Sec │  │                  │  ...markdown...      │
│             │  │ ▶ 300 · Tech │  │                  │                      │
│             │  └──────────────┘  │                  │  Sources             │
│             │                    │                  │  🌐 https://...      │
│             │                    │                  │     captured 3d ago  │
│             │                    │                  │                      │
│             │                    │                  │  Related             │
│             │                    │                  │  → 200.30            │
│             │                    │                  │                      │
│             │                    │                  │  [Research More]     │
└─────────────┴────────────────────┴──────────────────┴──────────────────────┘
```

### Mobile — Entry list (no slug)

```
┌─────────────────────────────┐
│ Knowledge                   │
├─────────────────────────────┤
│ 🔍 Search...                │
├─────────────────────────────┤
│ All | 100 | 200 | 300 | ... │  ← horizontal scroll tabs
├─────────────────────────────┤
│ JWT Auth Pattern      [404] │
│ 5d ago                      │
├─────────────────────────────┤
│ Philosophy Overview  [aged] │
│ 12d ago                     │
├─────────────────────────────┤
│ ...                         │
└─────────────────────────────┘
│ Work | Plan | Setup | Proj  │  ← BottomTabBar
└─────────────────────────────┘
```

## Component Tree

```
KnowledgePage
├── CatalogPane (w-72, hidden on mobile when slug present)
│   ├── SearchBar (text input, debounced 300ms)
│   └── CatalogTree
│       ├── AllRow (clears filter)
│       └── CatalogClassRow (expandable)
│           └── CatalogDivisionRow (indented, clickable filter)
├── EntryListPane (w-80, hidden on mobile when slug present)
│   ├── EntryRow (per entry)
│   │   ├── title + code badge
│   │   ├── StaleBadges (url_404, aged_out, code_ref_gone)
│   │   └── relative date
│   └── EmptyState / LoadingSkeleton
└── EntryDetailPane (flex-1, hidden on mobile when no slug)
    ├── DetailHeader
    │   ├── BackButton (mobile only)
    │   ├── title h2 + code badge
    │   └── StatusBadge
    ├── ContentSection (markdown render)
    ├── SourcesSection (provenance list)
    ├── RelatedSection (clickable code links)
    └── ResearchMoreButton (POST /api/knowledge/:slug/research)
```

## State Model

```typescript
// Page-level state
selectedCode: string | null     // catalog filter — null = All
searchQuery: string             // debounced search input
entries: KnowledgeEntrySummary[] // entry list from API
catalog: KnowledgeCatalog | null  // catalog tree from API
expandedClasses: Set<string>    // which catalog classes are expanded

// Detail pane state (per slug)
entry: KnowledgeEntryDetail | null
loading: boolean
researching: boolean            // Research More in-flight
```

## Data Flow

```
mount
  → fetch /api/knowledge/catalog  → catalog state
  → fetch /api/knowledge          → entries state (all)

selectedCode changes
  → fetch /api/knowledge?code=<prefix>  → entries state

searchQuery changes (debounced)
  → client-side filter of entries on title + summary
    (no server search endpoint in v1)

slug param changes
  → fetch /api/knowledge/:slug    → entry state

Research More click
  → POST /api/knowledge/:slug/research
  → set researching=true, await, set researching=false
  → show inline status "Research queued" / error message

SSE onUpdate
  → re-fetch /api/knowledge (and /api/knowledge/:slug if open)
```

## Staleness Badge Colors

| Flag | Label | Style |
|---|---|---|
| `url_404` | 404 | `bg-destructive/20 text-destructive text-xs px-1.5 rounded` |
| `aged_out` | aged | `bg-amber-500/20 text-amber-600 text-xs px-1.5 rounded` |
| `code_ref_gone` | stale ref | `bg-orange-500/20 text-orange-600 text-xs px-1.5 rounded` |

## Source Type Icons / Labels

| `type` value | Display |
|---|---|
| `web` | Globe icon + URL as link |
| `local_file` | File icon + path as monospace |
| `manual` | Pencil icon + "Manual" |
| `harvested` | Layers icon + `workspace` value |
| `guideline` | BookMarked icon + workspace value |

## File Changes

| File | Change |
|---|---|
| `frontend/src/pages/KnowledgePage.tsx` | New file — full page component |
| `frontend/src/lib/types.ts` | Add `KnowledgeCatalog*`, `KnowledgeEntry*`, `KnowledgeSseEvent` types |
| `frontend/src/api/client.ts` | Add `getKnowledgeCatalog`, `listKnowledge`, `getKnowledgeEntry`, `researchKnowledge` |
| `frontend/src/components/layout/Sidebar.tsx` | Add Knowledge nav item (Library icon, plan group, below Guidelines) |
| `frontend/src/components/layout/BottomTabBar.tsx` | Add `/knowledge` to Plan tab roots |
| `frontend/src/App.tsx` | Add `/knowledge` and `/knowledge/:slug` routes |

## Reuse Strategy

- `StatusBadge` — reused as-is for `draft` / `published`
- `Skeleton` — reused for loading placeholder rows
- `cn` utility — standard throughout
- `useSSE` hook — `onUpdate` callback for re-fetch
- `useParams` / `useNavigate` — react-router, same as all other pages
- No new shared components needed for v1 — all sub-components are local to `KnowledgePage.tsx`

## Markdown Rendering

The app does not currently have a shared Markdown renderer. For this page, use a simple `<pre className="whitespace-pre-wrap font-sans text-sm">` render of the raw content string. This is intentionally minimal — a follow-up feature can add a proper Markdown renderer when needed across multiple pages.

## Empty States

| Condition | Message |
|---|---|
| Catalog empty / not initialized | "Knowledge base not initialized. Run `sdlc knowledge librarian init` to seed from your project." |
| No entries for selected code | "No entries in this category." |
| Search returns no matches | "No entries match your search." |
| No entry selected (desktop) | Library icon + "Select an entry to view details" |

## Responsive Behavior

- Desktop (md+): three columns always visible
- Mobile: two-pane toggle — left+center pane OR detail pane based on slug presence
  - Mobile left/center pane: catalog tree collapses to a horizontal scrollable row of class code chips above the entry list to save vertical space
  - This avoids a three-panel layout that would be unusable on small screens
