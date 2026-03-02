# Feedback Improvements — Spec

## Problem

The feedback note queue is append-only and read-only after creation. Two gaps:

1. **No edit** — users can't fix typos or refine a note. Delete-and-recreate loses the ID.
2. **No enrichment** — notes arrive as raw quick-captures with no way to attach research context before submitting to ponder.

---

## Improvement 1: Edit notes

**Data model** (`feedback.rs`):
```rust
pub fn update(root: &Path, id: &str, new_content: impl Into<String>) -> Result<FeedbackNote>
```

**Server route**: `PATCH /api/feedback/:id` → `{ content: string }` body → returns updated note

**UX** (`NoteCard`):
- Click the note content area → transitions to textarea (same width/padding)
- ⌘+Enter saves, Escape cancels, clicking outside cancels
- Save triggers `api.updateFeedbackNote(id, content)` and refreshes state optimistically

---

## Improvement 2: Note enrichments

**Data model** — new `Enrichment` struct:
```rust
pub struct Enrichment {
    pub source: String,    // "user", "agent:research", "mcp:fetch"
    pub content: String,
    pub added_at: DateTime<Utc>,
}
```

Add `enrichments: Vec<Enrichment>` to `FeedbackNote` (serde default = empty vec for backward compat).

New function:
```rust
pub fn enrich(root: &Path, id: &str, source: &str, content: &str) -> Result<FeedbackNote>
```

**Server route**: `POST /api/feedback/:id/enrich` → `{ content: string, source: string }` → returns updated note

**UX** (`NoteCard`):
- "Add context" button (visible on hover, `Plus` icon)
- Expands inline textarea below the note content
- On save: `api.enrichFeedbackNote(id, content, "user")`
- Enrichment blocks rendered below a thin divider — muted background, smaller text, source tag

**`to_markdown` update**:
- Enrichments included below each note's content, labelled by source

**MCP tool** (future V2):
- `feedback_enrich` registered in sdlc MCP server
- External agents call it with `{ id, content, source: "agent:research" }`

---

## Phasing

| Phase | Scope |
|---|---|
| V1 | Edit + manual enrichment (user-authored context) |
| V2 | Agent-driven enrichment (spawn_agent_run + MCP tool) |

## Features to create

- `feedback-edit` — edit + PATCH route + inline UX
- `feedback-enrich` — Enrichment sub-model + enrich endpoint + card UX