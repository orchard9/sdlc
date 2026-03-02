# Spec: Enrichments — Attach Research Context to Feedback Notes

## Problem

Feedback notes are raw quick-captures. Before submitting a batch to the ponder workspace, users often want to attach additional context — a URL, a clarifying paragraph, a research finding — without replacing the original note. Today there is no mechanism to do this; the only option is delete-and-recreate, which loses the original ID and timestamp.

## Goal

Add an `Enrichment` sub-model to `FeedbackNote`. Users can attach one or more enrichment blocks to any note. Enrichments are surfaced in the UI (inline below the note), in the ponder markdown export, and (in a future V2) via an MCP tool for agent-authored context.

## Scope (V1 — this feature)

- `Enrichment` struct in `sdlc-core/src/feedback.rs`
- `enrichments: Vec<Enrichment>` field on `FeedbackNote` (serde `default` for backward compat)
- `feedback::enrich(root, id, source, content)` function
- `POST /api/feedback/:id/enrich` server route
- `api.enrichFeedbackNote(id, content, source)` frontend API client method
- `FeedbackNote.enrichments` field added to frontend TypeScript type
- `NoteCard` UI: "Add context" affordance + enrichment block rendering
- `to_markdown` updated to include enrichment blocks

## Out of Scope

- MCP tool (`feedback_enrich`) — V2
- Agent-driven enrichment via `spawn_agent_run` — V2
- Editing existing enrichments — future

## Data Model

### Rust (`sdlc-core/src/feedback.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrichment {
    pub source: String,       // e.g. "user", "agent:research", "mcp:fetch"
    pub content: String,
    pub added_at: DateTime<Utc>,
}

pub fn enrich(root: &Path, id: &str, source: &str, content: &str) -> Result<FeedbackNote>
```

### Server route

`POST /api/feedback/:id/enrich`
- Body: `{ content: string, source: string }`
- Returns: updated note JSON with `enrichments` array
- 404 if note not found

### Frontend TypeScript

```ts
export interface Enrichment {
  source: string
  content: string
  added_at: string
}
```

### API Client

```ts
enrichFeedbackNote: (id: string, content: string, source: string) =>
  request<FeedbackNote>(`/api/feedback/${encodeURIComponent(id)}/enrich`, {
    method: 'POST',
    body: JSON.stringify({ content, source }),
  }),
```

## UI Behaviour

- On hover, an "Add context" button appears using a `Plus` icon.
- Clicking expands an inline textarea (no modal).
- Cmd+Enter saves; Escape cancels.
- Enrichment blocks rendered below a thin divider with muted background, smaller text, left border accent.
- Source tag displayed as a pill.

## Acceptance Criteria

1. `feedback::enrich` adds an enrichment to an existing note and persists it.
2. `POST /api/feedback/:id/enrich` returns the updated note with the enrichment included.
3. Notes loaded without enrichments (old YAML) deserialize with `enrichments: []`.
4. "Add context" button appears on hover in `NoteCard`.
5. Submitting an enrichment updates the card in place without a full page reload.
6. `to_markdown` output includes enrichment blocks for each note that has them.
7. All existing feedback tests continue to pass.
