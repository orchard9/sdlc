# Tasks: Backlog section in Dashboard frontend

## Implementation Tasks

### T1 — Fetch /api/backlog in Dashboard.tsx and add Backlog section with open item count

Add the `/api/backlog` GET endpoint to `routes/backlog.rs` (with `list_backlog` handler), register it in `lib.rs` and `routes/mod.rs`. Add `getBacklog` to `api/client.ts`. Add `BacklogItem`/`BacklogKind`/`BacklogStatus` types to `types.ts`. In `Dashboard.tsx`, fetch backlog on mount and render a section header showing "Backlog" + open item count badge. Section appears above PreparePanel.

**Acceptance**: GET `/api/backlog` returns `[]` when no `backlog.yaml` exists; returns items array when file exists. Dashboard renders "Backlog" heading with `N open` badge.

---

### T2 — Render open backlog items list with inline Park and Promote action buttons

For each open backlog item render a card showing: title text, kind badge (color-coded: concern=red, idea=blue, debt=amber), `source_feature` link → `/features/<slug>` if set, relative `created_at` timestamp, staleness indicator (Clock icon + "stale" label) when item is older than 60 days. Include "Park" and "Promote" buttons per card.

Add source-feature filter dropdown above the list (appears only when 2+ distinct source features exist). Add "Show parked (N)" toggle below the list that reveals parked items read-only.

**Acceptance**: Items render with correct badge colors. Filter dropdown appears only with 2+ sources. Parked items hidden by default.

---

### T3 — Promote action creates feature and removes item from open list via re-fetch

Add `POST /api/backlog/:id/promote` handler in `routes/backlog.rs`. Body: `{ slug, title, description? }`. Handler: (1) calls `sdlc_core::feature::Feature::create`, (2) calls `BacklogStore::mark_promoted`. Returns `{ promoted_to: slug }`. On 409 (slug collision) returns error JSON.

Add `promoteBacklogItem` to `api/client.ts`.

In `Dashboard.tsx`: clicking "Promote" expands an inline form (slug auto-derived from title, editable; title pre-populated; optional description). Submit calls promote API. On success: re-fetch open backlog, show toast (bottom-right, auto-dismisses after 4s) with "Feature created → [slug]" link. On 409: show inline error "A feature with that slug already exists. Choose a different name."

**Acceptance**: Promote creates feature dir in `.sdlc/features/`. Item disappears from open list. Toast appears with link.

---

### T4 — [user-gap] Add empty state copy: 3-sentence explanation of origin, meaning, and non-blocking nature

When no open backlog items exist, render the exact copy:

> "When an agent finishes a run, it captures concerns that were out of scope for that feature. Items appear here. Promote to create a tracked feature, or Park to set aside. Nothing in the backlog blocks your current work."

**Acceptance**: Copy appears verbatim when open item list is empty. Does not appear when items are present.

---

### T5 — [user-gap] Show kind badge + source_feature link + timestamp on each item card

Ensure each item card shows the complete provenance trifecta: kind badge (colored), `source_feature` as a clickable link to `/features/<slug>` (or omitted when null), and `created_at` as a relative human-readable timestamp (e.g. "2 days ago", "3 hours ago").

**Acceptance**: All three elements visible on a card that has all fields set. Source feature link is omitted (not broken/empty) when `source_feature` is null.

---

### T6 — [user-gap] Promote CTA tooltip: 'Creates a draft feature. No work begins automatically.'

The Promote button/CTA must show a tooltip on hover with the exact text: "Creates a draft feature. No agent work begins automatically."

**Acceptance**: Tooltip visible on hover. Exact wording matches.

---

### T7 — [user-gap] Park CTA prompts for park_reason note in a modal before confirming

Add `POST /api/backlog/:id/park` handler in `routes/backlog.rs`. Body: `{ park_reason }`. Calls `BacklogStore::park`. Returns updated item or 422 on empty reason.

Add `parkBacklogItem` to `api/client.ts`.

In `Dashboard.tsx`: clicking "Park" expands an inline textarea ("Why park this item?"). Confirm button disabled until textarea is non-empty. On submit calls park API. On success: re-fetch open backlog (item disappears, parked count increments). On error: inline error below textarea.

**Acceptance**: Confirm disabled with empty textarea. Non-empty reason submits and item leaves open list. Server-side 422 on empty reason is also tested.

---

### T8 — [user-gap] Add source_feature filter dropdown — batch triage by feature origin

Derive distinct `source_feature` values from the fetched open backlog items. When 2 or more distinct values exist, render a `<select>` dropdown above the items list with "All sources" as default and one option per slug. Filtering is client-side only (no additional API call). Changing selection instantly filters visible items.

**Acceptance**: Dropdown absent with 0 or 1 source features. Dropdown present with 2+. Selecting a source shows only items with that source feature.

---

### T9 — [user-gap] Add staleness indicator for open items older than 60 days

For open items where `(Date.now() - new Date(item.created_at).getTime()) > 60 * 24 * 60 * 60 * 1000`, render a `<Clock>` icon (lucide-react) and "stale" label in muted amber color before the timestamp. Visual signal only — no automatic action.

**Acceptance**: Items < 60 days show no stale indicator. Items > 60 days show amber Clock + "stale" label.

---

## Rust Tests (part of T1/T3/T7)

- `list_empty_when_no_backlog_file` — GET `/api/backlog` → `[]`
- `list_returns_open_items` — seed via `BacklogStore::add`, GET returns them
- `park_item_updates_status` — POST `/api/backlog/B1/park` → item.status = parked
- `park_empty_reason_returns_422` — POST with empty reason → 422
- `promote_item_creates_feature_and_marks_promoted` — POST `/api/backlog/B1/promote` → feature dir created, item.promoted_to set

## Dependency order

T1 must be done first (backend + type definitions needed by all other tasks). T3 and T7 add the remaining two backend endpoints. T2, T4, T5, T6, T8, T9 are pure frontend and can be done in any order after T1.
