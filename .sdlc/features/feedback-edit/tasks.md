# Tasks: Edit Feedback Notes Inline

## T1 — Add `updated_at` to `FeedbackNote` and implement `update()` in sdlc-core

**File:** `crates/sdlc-core/src/feedback.rs`

- Add `#[serde(default)] pub updated_at: Option<DateTime<Utc>>` to `FeedbackNote`
- Implement `pub fn update(root: &Path, id: &str, content: impl Into<String>) -> Result<Option<FeedbackNote>>`
  - Loads all notes, finds by id, sets `content` and `updated_at = Some(Utc::now())`, saves, returns updated note
  - Returns `Ok(None)` if note not found
- Add unit tests:
  - `update_existing_note` — verify content and `updated_at` are set
  - `update_missing_returns_none`
  - `existing_notes_without_updated_at_deserialise` — write raw YAML without the field, confirm load succeeds

## T2 — Add `PATCH /api/feedback/:id` route in sdlc-server

**Files:** `crates/sdlc-server/src/routes/feedback.rs`, `crates/sdlc-server/src/lib.rs`

- Add `UpdateBody { content: String }` deserializable struct
- Add `update_note` async handler:
  - Returns 400 if `content.trim().is_empty()`
  - Calls `sdlc_core::feedback::update` in `spawn_blocking`
  - Returns 404 if result is `None`, 200 with updated note JSON otherwise
- Register route in `lib.rs`: `.route("/api/feedback/{id}", patch(routes::feedback::update_note))`
- Add integration tests in `routes/feedback.rs`:
  - `update_existing_note_returns_200`
  - `update_missing_note_returns_404`
  - `update_with_empty_content_returns_400`

## T3 — Add `updated_at` to frontend `FeedbackNote` type and `updateFeedbackNote` API method

**Files:** `frontend/src/lib/types.ts`, `frontend/src/api/client.ts`

- In `types.ts`: add `updated_at: string | null` to `FeedbackNote` interface
- In `client.ts`: add `updateFeedbackNote(id: string, content: string)` method — `PATCH /api/feedback/:id` with `{ content }` body

## T4 — Implement inline edit UI in `NoteCard` component

**File:** `frontend/src/pages/FeedbackPage.tsx`

- `FeedbackPage` gains two callbacks: `onEdit(id, newContent)` and `onEditError(id, originalContent)` passed to each `NoteCard`
- `onEdit` optimistically updates `notes` state; fires `api.updateFeedbackNote` in background; calls `onEditError` on network failure
- `NoteCard` gains local state: `editing`, `editDraft`, `editError`, `saving`
- Enter edit mode on double-click of the card body, or single-click of the pencil icon (`Pencil` from lucide-react) that appears on hover
- Edit mode renders:
  - `<textarea>` auto-focused, pre-filled with current content, auto-resizes to content
  - Save button (disabled when `editDraft.trim()` is empty or `saving`)
  - Cancel button
- Keyboard: `Cmd/Ctrl+Enter` saves, `Escape` cancels
- On save: call parent `onEdit`, close edit mode
- On cancel / Escape: restore `editDraft` to `note.content`, close edit mode
- Metadata line shows `· edited <timestamp>` when `note.updated_at` is non-null
- If another card opens edit mode (tracked via a parent-level `editingId` state), close the current one without saving

## T5 — Run tests and clippy; verify no regressions

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
cd frontend && npx tsc --noEmit
```

All must pass clean before this task is complete.
