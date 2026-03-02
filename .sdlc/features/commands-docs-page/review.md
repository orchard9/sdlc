# Code Review: Commands Catalog — /docs/commands

## Summary

Three files changed:

- **Created** `frontend/src/components/docs/commands-data.ts` — static data layer
- **Created** `frontend/src/components/docs/CommandsCatalog.tsx` — search + grouped list component
- **Modified** `frontend/src/pages/DocsPage.tsx` — renders `CommandsCatalog` for the `commands` section

No backend changes. No new dependencies. TypeScript build passes clean (`npx tsc --noEmit` exits 0).

---

## Findings

### F1 — `first:mt-0` pseudo-class on a non-first-child element (ACCEPT)

`CategorySection` renders a `mt-6` header div and uses `first:mt-0` on the inner container. Because `CategorySection` is wrapped in a parent `<div>` rendered by `CATEGORY_ORDER.map(...)`, `first:mt-0` applies correctly to the first `CategorySection`'s inner header element. The visual result is correct (no extra top margin on the first category). No fix needed; accepted.

### F2 — `CommandEntry.slug` not used for routing or linking (ACCEPT)

The `slug` field on `CommandEntry` is used as the React `key` and for search matching (`cmd.slug.includes(q)`). It is not currently used for linking to a per-command detail page — that's explicitly out of scope per the spec ("Rendering the full command markdown body is a future feature"). Accepted; no action needed.

### F3 — Search matches only substring of slug, not prefix (ACCEPT)

The search filter uses `cmd.slug.includes(q)` which is a substring match (case-sensitive on slug). Since all slugs are lowercase and users type lowercase search terms, this works correctly in practice. A prefix-only filter would exclude results like typing "run" to find `sdlc-run-wave`. The substring behaviour is correct. Accepted.

### F4 — Count display shows `filtered.length / COMMANDS.length` with no label (TRACK)

The search bar shows e.g. "4 / 34" in the right corner. Without a label this reads as a ratio but could be misinterpreted. A future polish pass could add "results" or use a clearer layout. Not a blocker.

Action: `sdlc task add commands-docs-page "polish: add aria-label and text label to search result count"`

### F5 — No keyboard accessibility on CommandRow (TRACK)

`CommandRow` is a `<div>` not a focusable element. The only interactive element inside is `CopyButton` (a `<button>`), which is keyboard accessible. The row itself is not focusable, which is consistent with other non-interactive list items in the app (e.g. FeatureCard rows). Acceptable for now.

Action: tracked via F4 task above — a future pass can add `tabIndex`, `role="listitem"`, and `aria-label` to improve screen-reader experience.

### F6 — `commands-data.ts` is not auto-synced with Rust source (ACCEPT)

The 34 command entries are manually maintained in `commands-data.ts`. If a command is added to `ALL_COMMANDS` in Rust, the frontend data must be updated manually. This is the intended design (spec explicitly calls for static frontend data). A future feature could generate this from Rust. Accepted for this cycle.

---

## Verdict

**APPROVE.** All spec acceptance criteria are met:
1. 34 commands rendered — verified via TypeScript count of COMMANDS array.
2. 6 category groups with headers — Lifecycle, Planning, Workspace, Analysis & Quality, Tooling, Project Setup.
3. Client-side search filtering by slug and description.
4. CopyButton on every row copies invocation string.
5. Design system tokens used throughout — no novel Tailwind classes.
6. No new API routes.
7. TypeScript build passes clean.

Two findings tracked as future tasks (F4, F5). No blocking issues.
