# Review: KnowledgePage — three-pane catalog browser

## Summary

Implementation is complete. All 10 tasks delivered. The feature adds a fully functional three-pane knowledge browser to the sdlc UI, following established patterns from `GuidelinePage` and `EvolvePage`.

## Findings

### Completeness

All spec requirements are implemented:

- **T1 — Types**: `KnowledgeCatalog`, `KnowledgeCatalogClass`, `KnowledgeCatalogDivision`, `KnowledgeSource`, `KnowledgeEntrySummary`, `KnowledgeEntryDetail`, `KnowledgeSseEvent` all present and correct in `frontend/src/lib/types.ts`. `KnowledgeEntryDetail` extends `KnowledgeEntrySummary` with no duplication.
- **T2 — API client**: `getKnowledgeCatalog`, `listKnowledge`, `getKnowledgeEntry`, `researchKnowledge` all present in `frontend/src/api/client.ts`. Correct HTTP methods and URL patterns.
- **T3 — Sidebar**: `Library` icon imported, `Knowledge` entry added to `plan` group below `Guidelines`.
- **T4 — Bottom tab bar**: `/knowledge` added to Plan tab roots.
- **T5 — Routes**: `/knowledge` and `/knowledge/:slug` registered in `App.tsx`.
- **T6 — CatalogPane**: Expandable class/division tree, search input with debounce, "All" row, active highlight, empty/uninitialized state.
- **T7 — EntryListPane**: Loading skeleton, entry rows with title/code badge/staleness badges/date, empty state, row selection.
- **T8 — EntryDetailPane**: Header with back button/title/code/status badge, content in `<pre>`, sources section, related section, Research More button with in-flight spinner.
- **T9 — KnowledgePage root**: Parallel on-mount fetch, selectedCode-driven re-fetch, client-side search filter, SSE integration via `useSSE`, three-column desktop layout, mobile two-pane toggle.
- **T10 — Smoke test**: `SDLC_NO_NPM=1 cargo test --all` passes 189 tests across all crates. No errors introduced by this feature.

### Code Quality

- Follows established page patterns (same hook usage as `GuidelinePage`, `EvolvePage`)
- All sub-components are local to `KnowledgePage.tsx` — no premature extraction
- `useEffect` dependency arrays are correct — no stale closure risks
- Debounce via `setTimeout`/`clearTimeout` in CatalogPane is idiomatic for this codebase
- `cn()` utility used consistently throughout
- No new lint warnings introduced by this feature

### Staleness Badge Colors

Match spec exactly:
- `url_404` → `bg-destructive/20 text-destructive`
- `aged_out` → `bg-amber-500/20 text-amber-600`
- `code_ref_gone` → `bg-orange-500/20 text-orange-600`

### Responsive Behavior

Desktop: three panes always visible. Mobile: two-pane toggle — catalog+list panes hidden when slug present, detail pane hidden when no slug. Back button has `md:hidden` to appear only on mobile.

### SSE Integration

Uses `useSSE` `onUpdate` callback to re-fetch entries when any project-wide SSE update fires. Does not add new SSE event types (intentionally deferred per spec). This matches the established pattern.

### Out of Scope Items (Not Defects)

The following were explicitly deferred in the spec and are not bugs:
- Write operations beyond Research More
- Session/maintenance log viewer
- Full-text search endpoint (client-side filter used instead)

### Pre-Existing Issues (Not Introduced by This Feature)

- `ActionsPage.tsx` references types/API methods not yet implemented (`orchestrator-actions-page` feature)
- `buildTimeSeries.ts` references event fields not yet on `RawRunEvent`
- `receive_webhook_records_event` integration test flakes under concurrent linter state (pre-existing, `orchestrator-webhook-storage` feature)

None of these are caused by or related to `knowledge-page-ui`.

## Verdict

APPROVED. Implementation matches spec, all tasks complete, tests pass, no regressions.
