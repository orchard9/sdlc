# QA Plan: KnowledgePage — three-pane catalog browser

## Build Verification

- [ ] `SDLC_NO_NPM=1 cargo test --all` passes with no failures
- [ ] `cd frontend && npm run build` exits 0 with no TypeScript errors
- [ ] `cargo clippy --all -- -D warnings` passes with no new warnings

## Types (T1)

- [ ] `KnowledgeCatalog`, `KnowledgeCatalogClass`, `KnowledgeCatalogDivision` exported from `types.ts`
- [ ] `KnowledgeEntrySummary`, `KnowledgeEntryDetail`, `KnowledgeSource` exported from `types.ts`
- [ ] `KnowledgeSseEvent` exported from `types.ts`
- [ ] `KnowledgeEntryDetail` extends `KnowledgeEntrySummary` (no field duplication)

## API Client (T2)

- [ ] `api.getKnowledgeCatalog()` calls `GET /api/knowledge/catalog`
- [ ] `api.listKnowledge()` calls `GET /api/knowledge`
- [ ] `api.listKnowledge('200')` calls `GET /api/knowledge?code=200`
- [ ] `api.getKnowledgeEntry('my-slug')` calls `GET /api/knowledge/my-slug`
- [ ] `api.researchKnowledge('my-slug')` calls `POST /api/knowledge/my-slug/research`

## Sidebar (T3)

- [ ] "Knowledge" nav item appears in the sidebar under the "plan" group, below "Guidelines"
- [ ] Uses `Library` icon from `lucide-react`
- [ ] Link navigates to `/knowledge`
- [ ] Nav item is highlighted (active) when on `/knowledge` or `/knowledge/:slug`

## Bottom Tab Bar (T4)

- [ ] "Plan" tab is active when pathname starts with `/knowledge`

## Routes (T5)

- [ ] Navigating to `/knowledge` renders `KnowledgePage`
- [ ] Navigating to `/knowledge/some-slug` renders `KnowledgePage` with detail pane
- [ ] No 404 or blank page at either route

## CatalogPane (T6)

- [ ] Search input is rendered at the top of the left pane
- [ ] Typing in the search input updates `searchQuery` (debounced)
- [ ] "All" row is highlighted when no code is selected and search is empty
- [ ] Clicking "All" clears the code filter
- [ ] Catalog class rows render with class `code` and `name`
- [ ] Clicking a class row toggles expansion of its divisions
- [ ] Division rows are indented under their parent class
- [ ] Clicking a division row sets `selectedCode` to that division's code and highlights the row
- [ ] When catalog is empty, shows "Knowledge base not initialized..." message

## EntryListPane (T7)

- [ ] Loading state shows three skeleton rows
- [ ] Entry rows show: title, code badge, staleness badges, relative date
- [ ] `url_404` flag renders red "404" badge
- [ ] `aged_out` flag renders amber "aged" badge
- [ ] `code_ref_gone` flag renders orange "stale ref" badge
- [ ] Entries with no staleness flags show no badges
- [ ] Clicking a row navigates to `/knowledge/<slug>`
- [ ] Selected row (matching URL slug) is highlighted
- [ ] Empty state shows "No entries found."
- [ ] Client-side search filter narrows list by `title` and `summary`

## EntryDetailPane (T8)

- [ ] Loading state shows centered spinner
- [ ] Error state shows "Entry not found" message
- [ ] Header shows: title, code badge, status badge
- [ ] Back button visible on mobile, hidden on desktop (md:hidden)
- [ ] Clicking back navigates to `/knowledge`
- [ ] Entry content rendered in a `<pre>` block
- [ ] Sources section shown when entry has sources
- [ ] Sources section omitted when entry has no sources
- [ ] Web source: shows URL as a clickable link
- [ ] Local file source: shows path in monospace
- [ ] Related section shown when `entry.related` is non-empty
- [ ] Related section omitted when `entry.related` is empty
- [ ] Clicking a related code/slug navigates to `/knowledge/<slug>`
- [ ] "Research More" button is present
- [ ] Clicking "Research More" shows a spinner and disables the button during the request
- [ ] After successful research request, button re-enables and shows confirmation message
- [ ] After failed research request, button re-enables and shows error message

## KnowledgePage root (T9)

- [ ] On mount, fetches catalog and entry list in parallel
- [ ] Changing `selectedCode` triggers a new fetch filtered by code
- [ ] SSE `onUpdate` triggers a re-fetch of the entry list
- [ ] Desktop: all three panes visible simultaneously
- [ ] Mobile with no slug: catalog + list panes visible, detail pane hidden
- [ ] Mobile with slug: detail pane visible, catalog + list panes hidden
- [ ] Navigating from one slug to another updates the detail pane

## Regression

- [ ] Guidelines page still works (no sidebar regression)
- [ ] Evolve page still works
- [ ] BottomTabBar Plan tab still activates for `/feedback`, `/ponder`, `/investigations`, `/evolve`, `/guidelines`
- [ ] No console errors on `/knowledge` or `/knowledge/:slug` load
