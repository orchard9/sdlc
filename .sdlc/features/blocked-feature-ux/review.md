# Code Review: Blocked Feature UX — BlockedPanel

## Summary

Implementation is complete across 5 files. ~160 lines of new code. All changes are
consistent with existing patterns. No new SSE variants, no modals, no cross-project lookup.

---

## File-by-file review

### `crates/sdlc-core/src/feature.rs`

**Method added:** `pub fn remove_blocker(&mut self, idx: usize) -> Result<()>`

**Assessment: APPROVED**

- Uses `SdlcError::InvalidPhase` for out-of-range index — acceptable reuse of existing
  error variant since no `InvalidIndex` variant exists and the HTTP mapping (400) is correct.
- Updates `self.updated_at` after mutation, consistent with all other mutation methods
  (`update_title`, `add_score`, `approve_artifact`, etc.).
- 3 unit tests added: removes correct element, out-of-range error, empty-list error. Full
  coverage of the happy path and both error cases.
- No `unwrap()` — uses `?` / explicit `Err` returns.

### `crates/sdlc-server/src/routes/features.rs`

**Handler added:** `pub async fn remove_blocker` + `RemoveBlockerBody`

**Assessment: APPROVED**

- Follows the exact same pattern as `merge_feature`, `transition_feature`: `spawn_blocking`
  wrapping a sync load/mutate/save cycle.
- `Option<Json<RemoveBlockerBody>>` makes the body truly optional — no body = no crash.
  Matches the design spec (reason is optional).
- Uses `add_comment` from `sdlc_core::comment` with `CommentFlag::Decision` and
  `CommentTarget::Feature` — correct for a blocker-removal rationale.
- `blocker_text` captured before removal so the comment body is meaningful.
- Import `sdlc_core::comment::{add_comment, CommentFlag, CommentTarget}` added correctly.
- No `unwrap()` in library-facing code.

**Potential improvement (not blocking):** The reason comment could include the blocker
text even when the blocker string is empty (defensive). Current code handles this correctly
via `unwrap_or_default()`.

### `crates/sdlc-server/src/lib.rs`

**Route registered:** `delete(routes::features::remove_blocker)` at
`/api/features/{slug}/blockers/{idx}`

**Assessment: APPROVED**

- Registered in correct order (after merge route, before milestones).
- Uses `delete()` handler — correct HTTP method for a destructive idempotent operation.
- Path pattern `{idx}` will be parsed as `usize` by Axum's `Path<(String, usize)>` extractor.

### `frontend/src/components/features/BlockedPanel.tsx`

**New component: ~130 lines**

**Assessment: APPROVED**

- Props interface is clean: `slug`, `blockers`, `allSlugs`, `isRunning`, `onRunWithDirection`.
- State management is minimal and correct: `removingIdx`, `reasons`, `submitting`, `direction`.
- `fetch` call to `DELETE /api/features/${slug}/blockers/${idx}` is correct. Body is
  only sent when reason is non-empty — consistent with spec.
- SSE auto-refresh: component correctly relies on the existing `Update` event → `useFeature`
  refetch. No polling, no manual refetch needed.
- In-project slug link detection: `allSlugs.includes(blocker)` — O(n) but feature lists
  are small in practice (<100). Acceptable.
- "Run with direction" disabled when `direction.trim() === ''` or `isRunning` — correct.
- `autoFocus` on reason input for good UX.
- Enter key on direction input calls `handleRunWithDirection` — matches common form UX.
- Amber color treatment is appropriate for a blocked/warning state.
- No emojis in UI text.

**Minor:** `key={idx}` on the blocker list uses numeric index. Since the blockers array
is not reordered from this component (only removed), index-as-key is acceptable here.

### `frontend/src/pages/FeatureDetail.tsx`

**Changes: imports + `allSlugs` state + conditional `BlockedPanel` render**

**Assessment: APPROVED**

- `useEffect` + `api.getFeatures()` for `allSlugs` is a simple one-time fetch. Uses
  `.catch(() => {})` to silently ignore failures — if the fetch fails, no links are shown
  but the component still renders, which is correct graceful degradation.
- `BlockedPanel` is rendered before the "Next action" card — correct positioning per spec.
- `onRunWithDirection` callback passes `context: direction` to `startRun` — uses the new
  `context?` field added to `StartRunOpts`.
- `runType: 'feature'` is correct.

### `frontend/src/contexts/AgentRunContext.tsx`

**Changes: `context?: string` field on `StartRunOpts`, body serialization in `startRun`**

**Assessment: APPROVED**

- `context?: string` is an optional field — all existing callers remain unaffected (they
  don't pass `context`, so body is `undefined`, matching previous behavior).
- `opts.context != null ? JSON.stringify(...) : undefined` — strictly correct null check
  (not falsy, so empty string would still be serialized, but callers use non-empty direction).

---

## Spec compliance check

| Acceptance criterion | Status |
|---|---|
| BlockedPanel visible when `feature.blocked === true` | Implemented |
| Each blocker listed; in-project slugs get link | Implemented |
| Remove button reveals inline reason; submits DELETE | Implemented |
| Blocker disappears after remove (SSE refresh) | Implemented (relies on existing Update event) |
| Direction + Run calls `/api/run/:slug` with `{ context }` | Implemented |
| "Run with direction" disabled when direction empty | Implemented |
| No new SSE variants | Confirmed |
| Build passes with no clippy warnings | Pre-existing unrelated failures in orchestrator/feedback modules; no errors in files modified by this feature |

---

## Issues found

None blocking. The pre-existing build failures (`feedback.rs`, `orchestrator/webhook.rs`)
are in-progress WIP changes on this branch, unrelated to this feature. No new issues
introduced.

---

## Verdict: APPROVED
