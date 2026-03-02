# Design: Enrichments — Attach Research Context to Feedback Notes

## Architecture

This feature is a pure additive data model extension. No new subsystems are introduced.

### Core Layer (`sdlc-core/src/feedback.rs`)

```
FeedbackNote
  ├── id: String
  ├── content: String
  ├── created_at: DateTime<Utc>
  ├── updated_at: Option<DateTime<Utc>>
  └── enrichments: Vec<Enrichment>  ← NEW (serde default for backward compat)

Enrichment
  ├── source: String    (e.g. "user", "agent:research")
  ├── content: String
  └── added_at: DateTime<Utc>
```

New function:
```rust
pub fn enrich(root: &Path, id: &str, source: &str, content: &str) -> Result<FeedbackNote>
```
- Load all notes
- Find by ID → error `SdlcError::FeedbackNoteNotFound` if missing
- Push `Enrichment { source, content, added_at: Utc::now() }`
- Atomic save
- Return updated note

### Error Handling

New error variant `SdlcError::FeedbackNoteNotFound(String)` maps to HTTP 404 in the server error handler.

### Server Layer (`sdlc-server/src/routes/feedback.rs`)

New handler: `POST /api/feedback/{id}/enrich`
- Body: `EnrichBody { content: String, source: String }`
- Spawns blocking task, calls `sdlc_core::feedback::enrich`
- Returns `note_to_json(note)` — the `note_to_json` helper is extended to include the `enrichments` array

`note_to_json` extension:
```json
{
  "id": "F1",
  "content": "...",
  "created_at": "...",
  "updated_at": null,
  "enrichments": [
    { "source": "user", "content": "...", "added_at": "..." }
  ]
}
```

### Frontend Layer

#### TypeScript types (`frontend/src/lib/types.ts`)
```ts
export interface Enrichment {
  source: string
  content: string
  added_at: string
}

export interface FeedbackNote {
  id: string
  content: string
  created_at: string
  updated_at?: string
  enrichments: Enrichment[]
}
```

#### API client (`frontend/src/api/client.ts`)
```ts
enrichFeedbackNote: (id: string, content: string, source: string) =>
  request<FeedbackNote>(`/api/feedback/${encodeURIComponent(id)}/enrich`, {
    method: 'POST',
    body: JSON.stringify({ content, source }),
  }),
```

#### UI (`frontend/src/pages/FeedbackPage.tsx` — NoteCard)

State per card:
- `enriching: boolean` — whether the textarea is open
- `enrichDraft: string` — current textarea value
- `enrichSaving: boolean` — spinner during API call
- `enrichError: string | null` — error message

Interaction flow:
1. Mouse enters card → show "Add context" button (Plus icon)
2. Click "Add context" → `setEnriching(true)`, focus textarea
3. Cmd+Enter → call `api.enrichFeedbackNote(note.id, enrichDraft, "user")` → update `notes` state with returned note → close textarea
4. Escape / blur → cancel, close textarea

Enrichment rendering (below note content, after `<hr>`):
```
┌─────────────────────────────────────────┐
│ [note content]                          │
│                                         │
│ ─────────────────────────────────────── │
│ ▌ [user] 2026-03-02 10:05              │
│ ▌ Enrichment content here...           │
└─────────────────────────────────────────┘
```

## Backward Compatibility

`#[serde(default)]` on `enrichments` ensures existing YAML files without the field deserialize to `Vec::new()`.

## Testing Strategy

- Unit tests in `sdlc-core/src/feedback.rs`: enrich adds enrichment, enrich missing returns error, multiple enrichments accumulate, old YAML deserializes cleanly
- Integration tests in `sdlc-server/src/routes/feedback.rs`: enrich returns updated note, enrich missing returns 404
