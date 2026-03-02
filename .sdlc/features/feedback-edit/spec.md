# Spec: Edit Feedback Notes Inline

## Problem

Feedback notes are immutable once saved. If a user makes a typo, wants to refine an idea, or needs to add detail to a saved note, their only option is to delete the note and re-type it from scratch. This is friction — especially for longer, multi-line notes.

## Goal

Allow users to edit the content of any existing feedback note directly inline in the FeedbackPage UI. Edits are persisted via a new `PATCH /api/feedback/:id` endpoint backed by an `update` function in `sdlc-core`.

## User Story

**As** a product team member capturing rough ideas in the Feedback inbox,
**I want** to click a note and edit its text in place,
**so that** I can refine a thought without deleting and re-entering it.

## Scope

### In scope

- Inline editing of `FeedbackNote.content` in the frontend `NoteCard` component
- New `update(root, id, new_content)` function in `crates/sdlc-core/src/feedback.rs`
- New `PATCH /api/feedback/:id` route in `crates/sdlc-server/src/routes/feedback.rs`
- New `updateFeedbackNote(id, content)` API client method in `frontend/src/api/client.ts`
- The `updated_at` timestamp on the note (see Data Model section) is set on every edit

### Out of scope

- Edit history / versioning
- Markdown preview in edit mode
- Bulk edit

## Interaction Design

**Trigger:** The user double-clicks a note card (or clicks a pencil icon that appears on hover).

**Edit mode:**
- The note's `<pre>` text renders as a resizable `<textarea>` pre-filled with the current content.
- Pressing `Enter` with `Cmd/Ctrl` saves the edit and returns to display mode.
- Pressing `Escape` cancels without saving.
- An explicit "Save" button is present alongside a "Cancel" button for discoverability.

**Save behaviour:**
- Optimistic update — the UI updates immediately and the PATCH request fires in the background.
- On network error the original content is restored and an error message is shown.

**No double-edit:** Clicking a different note while one is open in edit mode closes the edit without saving (same behaviour as Escape).

## Data Model

`FeedbackNote` gains an optional `updated_at` field:

```rust
pub struct FeedbackNote {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,  // None = never edited
}
```

This field is `Option` so existing YAML files without the field deserialise correctly. The frontend `FeedbackNote` interface in `lib/types.ts` mirrors this.

## API Contract

```
PATCH /api/feedback/:id
Content-Type: application/json
{ "content": "<new text>" }

200 OK
{ "id": "F1", "content": "...", "created_at": "...", "updated_at": "..." }

404 Not Found — note with that ID does not exist
400 Bad Request — content is empty or missing
```

## Acceptance Criteria

1. Double-clicking a note card opens it in edit mode with the existing content pre-filled.
2. Saving a non-empty edit persists to disk and reflects immediately in the UI without a page reload.
3. Pressing Escape in edit mode restores the original content with no API call.
4. Attempting to save an empty note is rejected (save button disabled, no API call).
5. A network failure during save shows an error and restores the pre-edit content.
6. The `PATCH /api/feedback/:id` endpoint returns 404 when the ID does not exist.
7. The `updated_at` field is set on every successful edit and visible in the NoteCard metadata line.
8. Existing notes without `updated_at` in YAML deserialise without error.
