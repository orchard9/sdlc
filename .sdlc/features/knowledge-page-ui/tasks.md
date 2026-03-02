# Tasks: KnowledgePage — three-pane catalog browser

## T1 — Add types to types.ts

Add the following TypeScript interfaces to `frontend/src/lib/types.ts`:
- `KnowledgeCatalogDivision` (`code`, `name`, `description?`)
- `KnowledgeCatalogClass` (`code`, `name`, `description?`, `divisions`)
- `KnowledgeCatalog` (`classes`, `updated_at`)
- `KnowledgeSource` (`type`, `url?`, `path?`, `workspace?`, `captured_at`)
- `KnowledgeEntrySummary` (`slug`, `title`, `code`, `status`, `summary?`, `tags`, `created_at`, `updated_at`)
- `KnowledgeEntryDetail extends KnowledgeEntrySummary` (adds `sources`, `related`, `origin`, `harvested_from?`, `last_verified_at?`, `staleness_flags`, `content`)
- `KnowledgeSseEvent` (`type: 'knowledge_research_completed' | 'knowledge_maintenance_completed'`, `slug?`)

## T2 — Add API client methods

Add to `frontend/src/api/client.ts`:
- `getKnowledgeCatalog()` → `GET /api/knowledge/catalog`
- `listKnowledge(code?: string)` → `GET /api/knowledge` or `GET /api/knowledge?code=<prefix>`
- `getKnowledgeEntry(slug: string)` → `GET /api/knowledge/:slug`
- `researchKnowledge(slug: string)` → `POST /api/knowledge/:slug/research`

## T3 — Add sidebar nav item

In `frontend/src/components/layout/Sidebar.tsx`:
- Import `Library` from `lucide-react`
- Add `{ path: '/knowledge', label: 'Knowledge', icon: Library, exact: false }` to the `plan` group, after the `Guidelines` entry

## T4 — Add bottom tab bar route

In `frontend/src/components/layout/BottomTabBar.tsx`:
- Add `'/knowledge'` to the `roots` array of the `Plan` tab

## T5 — Register routes in App.tsx

In `frontend/src/App.tsx`:
- Import `KnowledgePage` from `@/pages/KnowledgePage`
- Add `<Route path="/knowledge" element={<KnowledgePage />} />`
- Add `<Route path="/knowledge/:slug" element={<KnowledgePage />} />`

## T6 — Implement CatalogPane component (local to KnowledgePage.tsx)

Inside `KnowledgePage.tsx`, implement a `CatalogPane` sub-component:
- Accepts props: `catalog: KnowledgeCatalog | null`, `selectedCode: string | null`, `onSelect: (code: string | null) => void`, `searchQuery: string`, `onSearch: (q: string) => void`
- Renders a search input at the top (debounced 300ms via `setTimeout` in `onChange`)
- Renders an "All" row that calls `onSelect(null)` — highlighted when `selectedCode === null` and `searchQuery === ''`
- Renders each `CatalogClass` as an expandable row (chevron icon toggles `expandedClasses` local state)
- Renders each division as an indented row under its expanded class — clicking calls `onSelect(division.code)`
- Active row (matching `selectedCode`) highlighted with `bg-accent text-accent-foreground`
- Empty/uninitialized state: text "Knowledge base not initialized. Run `sdlc knowledge librarian init` to seed."

## T7 — Implement EntryListPane component (local to KnowledgePage.tsx)

Inside `KnowledgePage.tsx`, implement an `EntryListPane` sub-component:
- Accepts props: `entries: KnowledgeEntrySummary[]`, `loading: boolean`, `selectedSlug: string | undefined`, `onSelect: (slug: string) => void`
- Loading state: three `<Skeleton>` placeholder rows
- Empty state: "No entries found." centered message
- Each entry row: title (truncated), code badge, staleness badges, relative `updated_at` date
- Staleness badge colors per design spec: `url_404` = red, `aged_out` = amber, `code_ref_gone` = orange
- Selected row: `bg-accent text-accent-foreground`
- Clicking a row calls `onSelect(slug)`

## T8 — Implement EntryDetailPane component (local to KnowledgePage.tsx)

Inside `KnowledgePage.tsx`, implement an `EntryDetailPane` sub-component:
- Accepts props: `slug: string`, `onBack: () => void`, `onRefresh: () => void`
- Fetches `GET /api/knowledge/:slug` on mount / slug change via `useEffect`
- Loading: centered `<Loader2>` spinner
- Error: centered "Entry not found" message
- Header: back button (mobile only), `h2` title, code badge (`font-mono`), `StatusBadge`
- Content section: `<pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">` render of `entry.content`
- Sources section (omit if no sources): each source shows type label/icon, url as `<a>` or path as `<code>`, `captured_at` date
- Related section (omit if empty): each related code/slug as a `<button>` that navigates to `/knowledge/<slug>`
- Research More button: calls `researchKnowledge(slug)`, shows spinner while in-flight, shows inline success/error message

## T9 — Implement KnowledgePage root component

In `frontend/src/pages/KnowledgePage.tsx`:
- `useParams` to get optional `slug`
- `useNavigate` for routing
- State: `catalog`, `entries`, `selectedCode`, `searchQuery`, `loading`
- On mount: fetch catalog and entries in parallel
- When `selectedCode` changes: re-fetch entries with `?code=<selectedCode>`
- When `searchQuery` changes (debounced): filter `entries` client-side on `title` and `summary`
- `useSSE(handleUpdate)` where `handleUpdate` re-fetches entries (and detail if slug is set)
- Three-column layout on desktop: `CatalogPane` (w-72) | `EntryListPane` (w-80) | `EntryDetailPane` (flex-1)
- Mobile: hide catalog+list panes when slug is present; hide detail pane when no slug

## T10 — Smoke test

After implementation:
- Run `SDLC_NO_NPM=1 cargo test --all` to verify server tests still pass
- Run `cd frontend && npm run build` to verify TypeScript compiles with no errors
- Verify no `cargo clippy` warnings introduced
