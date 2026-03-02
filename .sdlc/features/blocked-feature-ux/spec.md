# Spec: Blocked Feature UX — BlockedPanel with Remove and Run-with-Direction

## Problem

When a feature has blockers (non-empty `feature.blockers` list), the current UI offers
no affordances for the user or agent to act on them. The feature detail page shows that
the feature is blocked, but provides no way to:

1. See which specific blockers are registered
2. Remove a blocker (optionally with a reason)
3. Give the agent a direction to advance despite the block

Agents encountering `unblock_dependency` as the next action have no UI path — they must
fall back to CLI commands. Users looking at a blocked feature have no clear call-to-action.

## Solution

Add a `BlockedPanel` component that renders at the top of the feature detail page whenever
`feature.blocked === true`. The panel presents:

- **Zone 1 — Blocker list**: Each blocker string is shown. If the string exactly matches a
  known in-project feature slug, it is rendered as a link `→ Go to [slug]`. Each blocker
  has a "Remove" button with an optional inline reason field.

- **Zone 2 — Direction input + Run button**: A text input where the user types direction
  ("skip auth-setup, use env vars"). A primary "Run with direction" button POSTs to the
  existing `/api/run/:slug` endpoint with `{ context: "<direction>" }`.

No new SSE variants are needed — the existing mtime file watcher emits `SseMessage::Update`
when the feature manifest changes, and the frontend re-fetches on `Update`.

## Backend changes

### `sdlc-core/src/feature.rs`

Add `pub fn remove_blocker(&mut self, idx: usize) -> SdlcResult<()>`:

- Validates that `idx` is in range.
- Removes the element at `idx` from `self.blockers`.
- Updates `self.updated_at`.

### `sdlc-server/src/routes/features.rs`

Add `DELETE /api/features/:slug/blockers/:idx` handler:

- Loads the feature.
- Calls `feature.remove_blocker(idx)`.
- Saves the feature.
- Optionally records a `decision` comment via the existing comment mechanism if a
  `reason` body field is provided.

### `sdlc-server/src/lib.rs`

Register the new DELETE route.

## Frontend changes

### `frontend/src/components/features/BlockedPanel.tsx` (new file)

A self-contained component accepting `{ feature: FeatureDetail, allSlugs: string[] }`.

Renders:

```
┌─ ⚠ Blocked ─────────────────────────────────────────────────┐
│  • "waiting for product-content"  [→ product-content]        │
│  • "auth-setup must complete"                                 │
│    [✕ Remove]  [reason: _________________ ] [Confirm]        │
│                                                               │
│  ─── Give the agent direction to advance ─────────────────── │
│  [text input: e.g. "skip auth-setup, use env vars"]          │
│                                                               │
│  [▶ Run with direction]                                       │
└───────────────────────────────────────────────────────────────┘
```

- Remove button is per-blocker; clicking reveals an inline optional reason field.
- Confirm sends `DELETE /api/features/:slug/blockers/:idx` with optional `{ reason }`.
- On success, the component relies on SSE `Update` to refresh; no manual refetch needed.
- "Run with direction" is disabled when direction input is empty.
- "Run with direction" calls `startRun` from `AgentRunContext` with `context` appended
  to the start URL body via the existing `POST /api/run/:slug` pattern.

### `frontend/src/pages/FeatureDetail.tsx`

Import `BlockedPanel` and render it at the top of the page body (after the header,
before "Next action") when `feature.blocked === true`.

Pass `allSlugs` as the list of all feature slugs (fetched from the existing features
list or extracted from a state query) so the panel can render slug links.

## What is explicitly out of scope

- No cross-project dependency navigation
- No new SSE variant (`BlockerResolved` or similar)
- No polling / "check now" button
- No modal for reason capture (inline is sufficient)
- No `sdlc feature unblock` CLI command in v1
- No "waive" terminology (blockers are removed, not waived)

## Acceptance criteria

1. When a feature has blockers, `BlockedPanel` is visible at the top of the feature
   detail page.
2. Each blocker is listed. If the text exactly matches an in-project feature slug, a
   link `→ [slug]` is shown.
3. Clicking "Remove" on a blocker reveals an inline reason input. Submitting calls
   `DELETE /api/features/:slug/blockers/:idx`.
4. After a successful remove, the blocker disappears from the list (via SSE refresh).
5. Typing into the direction field and clicking "Run with direction" starts an agent run
   via `/api/run/:slug` with `{ context: "<direction>" }`.
6. "Run with direction" is disabled when the direction input is empty.
7. No new SSE variants are introduced.
8. Build passes with no `clippy` warnings.
