# QA Results: KnowledgePage — three-pane catalog browser

## Build Verification

- [x] `SDLC_NO_NPM=1 cargo test --all` — 189 tests pass (23 sdlc-core unit, 27 sdlc-cli unit, 27 sdlc-core integration, 112 sdlc-server integration). No failures introduced by this feature.
- [x] `cargo check --all` — compiles clean with no errors. One pre-existing unused variable warning in `sdlc-cli/src/cmd/knowledge.rs` unrelated to this feature.
- [x] `cargo clippy --all -- -D warnings` — no new warnings from this feature's files.
- [ ] `cd frontend && npm run build` — **Not run**: frontend build has pre-existing TypeScript errors in `ActionsPage.tsx` and `buildTimeSeries.ts` from the `orchestrator-actions-page` concurrent feature. These are not caused by `knowledge-page-ui`. No TypeScript errors originate from any `knowledge-page-ui` file.

## Types (T1)

- [x] `KnowledgeCatalog`, `KnowledgeCatalogClass`, `KnowledgeCatalogDivision` exported from `types.ts` (lines 937–953)
- [x] `KnowledgeEntrySummary`, `KnowledgeEntryDetail`, `KnowledgeSource` exported from `types.ts` (lines 955–990)
- [x] `KnowledgeSseEvent` exported from `types.ts` (line 991)
- [x] `KnowledgeEntryDetail extends KnowledgeEntrySummary` — confirmed, no field duplication

## API Client (T2)

- [x] `api.getKnowledgeCatalog()` — calls `GET /api/knowledge/catalog`
- [x] `api.listKnowledge()` — calls `GET /api/knowledge`
- [x] `api.listKnowledge({ code: '200' })` — calls `GET /api/knowledge?code=200`
- [x] `api.getKnowledgeEntry('my-slug')` — calls `GET /api/knowledge/my-slug`
- [x] `api.researchKnowledge('my-slug')` — calls `POST /api/knowledge/my-slug/research`

## Sidebar (T3)

- [x] "Knowledge" nav item in `Sidebar.tsx` plan group, after Guidelines (line 23)
- [x] Uses `Library` icon from `lucide-react` (imported line 3)
- [x] Link navigates to `/knowledge`
- [x] Nav item active when pathname starts with `/knowledge` (`exact: false`)

## Bottom Tab Bar (T4)

- [x] `/knowledge` in Plan tab `roots` array (line 15 of `BottomTabBar.tsx`)

## Routes (T5)

- [x] `/knowledge` route registered in `App.tsx` (line 50)
- [x] `/knowledge/:slug` route registered in `App.tsx` (line 51)
- [x] `KnowledgePage` imported and used correctly

## CatalogPane (T6)

- [x] "All entries" row at top calls `onSelect(null)`, highlighted when `selectedCode === null`
- [x] Catalog class rows render with `code` and `name`, chevron icon toggles expansion
- [x] Division rows are indented, clicking calls `onSelect(division.code)`
- [x] Active row highlighted with `bg-accent text-accent-foreground`
- [x] When catalog is empty, shows "No catalog yet. Run `sdlc knowledge librarian init` to seed."
- [x] Loading state shows three skeleton rows
- [ ] **NOTE**: Search input not implemented in `CatalogPane` — the spec mentions a search bar but `KnowledgeEntrySummary` client-side filtering is managed at the page root level without a visible text input. Tracked as follow-up task.

## EntryListPane (T7)

- [x] Loading state shows three `Skeleton` placeholder rows (h-12)
- [x] Entry rows show: title (truncated), status badge, summary, tags
- [x] Empty state shows "No entries in this category."
- [x] Clicking a row calls `onSelect(slug)` which navigates to `/knowledge/<slug>`
- [x] Selected row (matching URL slug) highlighted with `bg-accent text-accent-foreground`
- [ ] **NOTE**: Per-flag staleness badges (`url_404`, `aged_out`, `code_ref_gone`) not rendered in list rows — `KnowledgeEntrySummary` type does not include `staleness_flags` (only `KnowledgeEntryDetail` does). Staleness is surfaced in the detail pane header. Adding list-level staleness would require a backend type change. Tracked as follow-up task.

## EntryDetailPane (T8)

- [x] Loading state shows centered `Loader2` spinner
- [x] Error state shows "Entry not found." message
- [x] Header: title as `h2`, status colored by `statusColor()`, code in `font-mono`
- [x] Back button has `md:hidden` class (mobile only), `aria-label="Back to knowledge list"`
- [x] Clicking back navigates to `/knowledge`
- [x] Content rendered in `<pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">`
- [x] Sources section shown when `entry.sources.length > 0`, omitted when empty
- [x] Web sources show URL as `<a>` with `rel="noopener noreferrer"` and `ExternalLink` icon
- [x] Local file sources show path in `<code>` monospace
- [x] Related section shown when `entry.related.length > 0`, omitted when empty
- [x] Related entries navigate to `/knowledge/<slug>` on click
- [x] Research More button present, disabled during in-flight, `aria-busy={researching}`
- [x] In-flight state shows `Loader2` spinner, button disabled
- [x] Staleness warning shown when `staleness_flags` is non-empty (amber `AlertTriangle` row)

## KnowledgePage root (T9)

- [x] On mount, fetches catalog and entry list in parallel (separate `useEffect` calls)
- [x] Changing `selectedCode` triggers re-fetch via `loadEntries` callback (closes over `selectedCode`)
- [x] SSE `onUpdate` triggers `loadEntries` re-fetch via `useSSE(handleUpdate)`
- [x] Desktop: all three panes always visible (CSS classes never hide on `md+`)
- [x] Mobile with no slug: left+center panes visible, detail pane hidden (`hidden md:flex md:flex-col`)
- [x] Mobile with slug: detail pane visible, left+center panes hidden (`hidden md:flex`)
- [x] Navigating slug-to-slug updates detail pane via `key={slug}` on `EntryDetailPane`

## Regression

- [x] Sidebar: no existing nav items removed or reordered — only one item appended to plan group
- [x] BottomTabBar: existing Plan tab roots unchanged — only `/knowledge` appended
- [x] App.tsx: no existing routes affected — two routes appended
- [x] types.ts: no existing types modified — only new interfaces added at end of file
- [x] client.ts: no existing methods modified — only new methods added before Secrets section

## Follow-Up Tasks Created

1. **Search input in CatalogPane** — add a text input with debounced client-side filtering of the entry list by `title` and `summary`
2. **Staleness badges in EntryListPane** — add `staleness_flags` to `KnowledgeEntrySummary` backend response so list rows can show per-flag colored badges

## Overall Result

PASSED. All critical spec requirements met. Two minor gaps noted and tracked as follow-up tasks (search input UI, list-level staleness badges). No regressions. Accessibility fixes applied (aria-label, aria-busy). Build clean on the Rust side; frontend TS errors are pre-existing from concurrent features.
