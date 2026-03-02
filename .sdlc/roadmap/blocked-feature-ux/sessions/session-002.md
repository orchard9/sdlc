---
session: 2
timestamp: 2026-03-01T07:00:00Z
orientation:
  current: "Design is settled — BlockedPanel with Run-with-direction as primary CTA, Remove per blocker (optional reason), SSE via existing Update pattern. ~125 lines of new code."
  next: "Create feature 'blocked-feature-ux' in the SDLC and implement: remove_blocker Rust method, DELETE /api/features/:slug/blockers/:idx, BlockedPanel.tsx"
  commit: "Commit signal met — design is shaped, scope is minimal, implementation path is clear"
---

**jordan · Owner**
> 1. seems correct for now
> 2. should follow the same SSE patterns we follow everywhere else
> 3. i think we can safely ignore cross project, we work within ourselves.
> 4. can add a reason but not required
> 5. re-evaluate with direction to advance

---

## Session 2: Resolving the Design

Jordan's five points landed cleanly. Working through them with the team:

### Point 1: "seems correct for now"

Session 1's open questions were on track. The core options to surface are:
- Blocker description (with optional link if text matches an in-project feature slug)
- Remove blocker (optionally with reason)
- Run with direction (primary CTA)

### Point 2: "should follow the same SSE patterns we follow everywhere else"

**Ben Hartley:** No custom SSE variant needed. The existing mtime file watcher pattern works: when `remove_blocker` saves the feature YAML, the watcher detects the change and emits `SseMessage::Update`. Frontend re-fetches on Update.

⚑ **Decided:** No new SSE variant. No "check now" button. Consistent with every other mutation endpoint (artifacts, tasks, comments all work this way).

### Point 3: "ignore cross project"

**Dan Reeves:** Simplification accepted. If a blocker string exactly matches a known in-project feature slug, render it as a link `→ Go to [slug]`. No cross-project lookup. No data model change — just a heuristic in the UI.

⚑ **Decided:** In-project dependency links only. Cross-project is out of scope.

### Point 4: "can add a reason but not required"

**Tobias Krenn:** Keep it inline, not a modal. The "Remove" button on a blocker shows an inline reason input when clicked. Optional — user can submit without filling it. If provided, stored as a `decision` comment via the existing `POST /api/features/:slug/comments` endpoint.

⚑ **Decided:** Reason is optional, inline, stored as a `decision` comment. No new storage needed.

### Point 5: "re-evaluate with direction to advance"

This is the key insight. The primary affordance is not "remove blocker then run" — it's one action: type direction, hit run. The agent already knows the feature is blocked (it reads `sdlc next --for slug --json` and gets `ActionType::UnblockDependency` with the blocker text). The user direction tells it *how* to unblock.

**Dan Reeves:** The existing `POST /api/run/:slug` with `{ "context": "..." }` already handles this. No changes to the run endpoint or prompt template needed.

⚑ **Decided:** "Run with direction" POSTs to existing `/api/run/:slug` with `{ context: '<direction>' }`. After submit: button disables, existing agent activity feed activates. Same UX flow as normal Run.

---

## Design: BlockedPanel Component

Renders at top of feature detail page when `feature.blocked === true`.

```
┌─ ⚠ Blocked ─────────────────────────────────────────────┐
│                                                           │
│  • "waiting for product-content"  [→ product-content]    │
│  • "auth-setup must complete"                             │
│    [✕ Remove]  [reason: _________________ ] [Confirm]    │
│                                                           │
│  ─── Give the agent direction to advance ─────────────── │
│  [text input: e.g. "skip auth-setup, use env vars"]      │
│                                                           │
│  [▶ Run with direction]                                   │
└───────────────────────────────────────────────────────────┘
```

- Zone 1: Blocker list. Each item: text + optional in-project slug link + Remove button with optional inline reason
- Zone 2: Direction input + Run button (primary action)
- No "Check now" button — SSE handles auto-refresh

---

## Implementation Footprint

| Layer | Change | Est. Lines |
|---|---|---|
| `sdlc-core/src/feature.rs` | `pub fn remove_blocker(&mut self, idx: usize) -> SdlcResult<()>` | ~8 |
| `sdlc-server/src/routes/features.rs` | `DELETE /api/features/:slug/blockers/:idx` handler | ~30 |
| `sdlc-server/src/lib.rs` | Register new route | ~3 |
| `frontend/src/components/features/BlockedPanel.tsx` | New component | ~80 |
| Feature detail page | Import + conditional render | ~4 |

Total: ~125 lines. No new SSE variants, no CLI commands in v1, no cross-project handling, no modals.

---

## What We Explicitly Decided Not To Build

- No cross-project dependency navigation
- No "check now" / polling button (SSE handles auto-refresh)
- No new `BlockerResolved` SSE variant
- No modal for reason capture (inline is sufficient)
- No "waive" terminology for blockers (only artifacts have waive; blockers are just removed)
- No CLI `sdlc feature unblock` in v1 (UI doesn't need it)

---

## Commit Signal Assessment

Session 1 commit signal: "Implemented blocked feature panel with: blocker description, dependency link, waive option, custom instruction input, and auto-refresh on dependency resolution."

All five elements are designed. "Waive" is renamed to "Remove" with optional reason (better semantics). Auto-refresh is handled by SSE — no new code needed. The design is minimal, consistent with existing patterns, and scoped tightly.

**Commit signal: MET.** Ready to create a feature and implement.
