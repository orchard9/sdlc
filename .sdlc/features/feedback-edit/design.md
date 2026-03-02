# Design: Edit Feedback Notes Inline

## Overview

Three-layer change: core data layer (`sdlc-core`), HTTP route (`sdlc-server`), and React UI (`frontend`). No new files are introduced — all changes extend existing modules.

## Layer 1: sdlc-core — `feedback.rs`

### Data model change

Add `updated_at: Option<DateTime<Utc>>` to `FeedbackNote`. The field is `Option` so existing YAML files without it deserialise via serde default.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackNote {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}
```

### New public function

```rust
/// Update the content of an existing note. Returns the updated note, or
/// `Ok(None)` if no note with `id` exists.
pub fn update(root: &Path, id: &str, content: impl Into<String>) -> Result<Option<FeedbackNote>> {
    let content = content.into();
    let mut notes = load_all(root)?;
    let Some(note) = notes.iter_mut().find(|n| n.id == id) else {
        return Ok(None);
    };
    note.content = content;
    note.updated_at = Some(Utc::now());
    let updated = note.clone();
    save_all(root, &notes)?;
    Ok(Some(updated))
}
```

## Layer 2: sdlc-server — `routes/feedback.rs` + `lib.rs`

### New handler

```rust
#[derive(serde::Deserialize)]
pub struct UpdateBody {
    pub content: String,
}

/// PATCH /api/feedback/:id — update a feedback note's content
pub async fn update_note(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.content.trim().is_empty() {
        return Err(AppError::bad_request("content cannot be empty"));
    }
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::feedback::update(&root, &id_clone, &body.content)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    match result {
        Some(note) => Ok(Json(note_to_json(&note))),
        None => Err(AppError::not_found(format!("feedback note '{id}' not found"))),
    }
}
```

### Route registration (lib.rs)

```rust
.route("/api/feedback/{id}", patch(routes::feedback::update_note))
```

This sits alongside the existing DELETE route for the same path.

## Layer 3: frontend

### `lib/types.ts` — add `updated_at`

```typescript
export interface FeedbackNote {
  id: string
  content: string
  created_at: string
  updated_at: string | null
}
```

### `api/client.ts` — add `updateFeedbackNote`

```typescript
updateFeedbackNote: (id: string, content: string) =>
  request<FeedbackNote>(`/api/feedback/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify({ content }),
  }),
```

### `pages/FeedbackPage.tsx` — `NoteCard` inline edit

`NoteCard` gains local state:

```typescript
const [editing, setEditing] = useState(false)
const [editDraft, setEditDraft] = useState(note.content)
const [editError, setEditError] = useState<string | null>(null)
const [saving, setSaving] = useState(false)
```

**Trigger:** `onDoubleClick` on the card body opens edit mode. A pencil icon (`Pencil` from lucide-react) appears on hover as an alternative trigger.

**Edit mode layout:** replaces the `<pre>` with an auto-focused `<textarea>`. Save/Cancel buttons appear below the textarea.

**Save flow (optimistic):**
1. Immediately call `onEdit(note.id, trimmedDraft)` to update the parent list.
2. Fire `api.updateFeedbackNote(note.id, trimmedDraft)` in the background.
3. On error: call `onEditError(note.id, note.content)` to restore original, set `editError`.

**Parent (`FeedbackPage`) changes:**
- `onEdit(id, content)` — optimistically updates `notes` state.
- `onEditError(id, originalContent)` — restores on error.
- No extra network round-trip needed; the optimistic update suffices.

**Metadata line:** shows `edited <timestamp>` when `updated_at` is set.

## ASCII Wireframe — NoteCard (edit mode)

```
┌─────────────────────────────────────────────────┐
│ ┌─────────────────────────────────────────────┐ │
│ │ Write anything — ideas, issues, …           │ │
│ │                                             │ │
│ │                                             │ │
│ └─────────────────────────────────────────────┘ │
│                          [Cancel]  [Save]        │
│ F3 · Mar 2, 10:14 AM · edited                   │
└─────────────────────────────────────────────────┘
```

## Error handling

| Scenario | Behaviour |
|---|---|
| Empty content on save | Save button is disabled; no API call |
| Network error on PATCH | Restore original content; show inline error below textarea |
| Note deleted concurrently | Server returns 404; UI shows error, note stays visible |
| Escape key | Cancel without save; no API call |

## No SSE needed

Feedback notes are not pushed via SSE today and inline edits do not require real-time multi-client sync. The optimistic update in the editing client is sufficient.
