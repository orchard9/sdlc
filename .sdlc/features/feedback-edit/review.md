# Code Review: Edit Feedback Notes Inline

## Summary

Three-layer change: `sdlc-core` data layer, `sdlc-server` HTTP routes, and React frontend. All changes are contained within existing modules — no new files created.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/feedback.rs` | Added `#[serde(default)] updated_at: Option<DateTime<Utc>>` to `FeedbackNote`; added `update()` function; unit tests cover success, missing ID, legacy YAML compat |
| `crates/sdlc-server/src/routes/feedback.rs` | Added `UpdateBody` struct, `update_note` handler, `updated_at` field in `note_to_json`; route tests for 200/404/400 |
| `crates/sdlc-server/src/lib.rs` | Registered `PATCH /api/feedback/{id}` |
| `frontend/src/lib/types.ts` | Added `updated_at: string \| null` to `FeedbackNote` interface |
| `frontend/src/api/client.ts` | Added `updateFeedbackNote(id, content)` method — `PATCH /api/feedback/:id` with `{ content }` body |
| `frontend/src/pages/FeedbackPage.tsx` | Full inline edit implementation in `NoteCard`; `editingId` lifted to parent; optimistic update flow |

## Correctness Review

### sdlc-core changes

- `update()` uses `iter_mut()` to find by ID, sets `content` and `updated_at = Utc::now()`, saves only if found — no unnecessary writes.
- Returns `Ok(None)` for missing IDs — no panic path.
- `#[serde(default)]` on `updated_at` ensures existing YAML files without the field deserialise via serde default (None).
- `add()` sets `updated_at: now` matching `created_at` on creation.
- Tests: `update_content`, `update_missing_returns_none`, `old_yaml_backward_compat_no_enrichments` (backward compat), plus all pre-existing tests pass.

### sdlc-server changes

- `update_note` validates `content.trim().is_empty()` and returns 400 before touching disk.
- Uses `spawn_blocking` pattern — consistent with all other handlers.
- Returns 404 via `AppError::not_found` when core returns `Ok(None)`.
- `note_to_json` includes `updated_at` — all list/add/update/enrich responses expose the field consistently.
- Route registered as separate `.route("/api/feedback/{id}", patch(...))` alongside existing `DELETE` — Axum handles per-method routing.
- Tests: `update_existing_note_returns_200`, `update_missing_note_returns_404`, `update_with_empty_content_returns_400`.

### Frontend changes

- `editingId` state lifted to `FeedbackPage` parent — only one card in edit mode at a time; opening note B closes note A without saving.
- `NoteCard` local state: `editDraft`, `editError`, `saving`.
- `useEffect` syncs `editDraft` from `note.content` only when `!isEditing` — avoids clobbering in-flight user input.
- `openEdit()`, `cancelEdit()`, `saveEdit()` are clearly separated with no state leakage.
- Optimistic update: `onEdit()` updates parent state immediately; `onEditError()` restores original on network failure.
- Save button disabled when `editDraft.trim()` is empty — no empty-content API calls fired.
- Keyboard: `Cmd/Ctrl+Enter` saves, `Escape` cancels — matches spec.
- Double-click on card body opens edit mode; pencil icon on hover provides alternative trigger.
- Metadata line shows `· edited <timestamp>` when `note.updated_at` is non-null.
- `Pencil` icon imported from `lucide-react`.

## Quality Checks

- `SDLC_NO_NPM=1 cargo test --all` — all 736 tests pass, zero failures.
- `cargo clippy --all -- -D warnings` — clean, zero warnings.
- `cd frontend && npx tsc --noEmit` — zero TypeScript errors.

## Issues Found

None — implementation matches the design and spec exactly. All acceptance criteria are satisfied.

## Verdict

APPROVED
