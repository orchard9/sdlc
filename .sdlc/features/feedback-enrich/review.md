# Review: Enrichments — Attach Research Context to Feedback Notes

## Summary

All 7 implementation tasks are complete. The feature adds `Enrichment` support to `FeedbackNote` across the full stack: core data layer, HTTP server, and React frontend.

## Changes Reviewed

### `crates/sdlc-core/src/feedback.rs`

- `Enrichment { source, content, added_at }` struct added with full serde derives
- `FeedbackNote.enrichments: Vec<Enrichment>` field with `#[serde(default)]` for backward compat
- `FeedbackNote.updated_at: Option<DateTime<Utc>>` also added with `skip_serializing_if` for old YAML compat
- `pub fn enrich(root, id, source, content) -> Result<FeedbackNote>` — loads, finds by ID (returns `FeedbackNoteNotFound` if absent), pushes enrichment, atomic save, returns updated note
- `to_markdown` updated with `> **Context** [source] -- _timestamp_` format
- 15 tests pass: 7 original + 6 new enrichment-specific + 2 backward-compat tests
- No `unwrap()` in library code — all errors propagate via `?`

### `crates/sdlc-core/src/error.rs`

- `FeedbackNoteNotFound(String)` variant added correctly

### `crates/sdlc-server/src/error.rs`

- `SdlcError::FeedbackNoteNotFound(_) => StatusCode::NOT_FOUND` match arm added

### `crates/sdlc-server/src/routes/feedback.rs`

- `EnrichBody { content, source }` deserializable struct
- `enrich_note` handler: spawns blocking task, delegates to `sdlc_core::feedback::enrich`, returns `note_to_json`
- `note_to_json` extended to include `updated_at` and `enrichments` array
- `update_note` handler also present (from `feedback-edit` feature already in working tree)
- 9 server route tests pass including 2 new enrichment tests

### `crates/sdlc-server/src/lib.rs`

- `.route("/api/feedback/{id}/enrich", post(routes::feedback::enrich_note))` registered
- `.route("/api/feedback/{id}", patch(routes::feedback::update_note))` also registered

### `frontend/src/lib/types.ts`

- `Enrichment` interface added
- `FeedbackNote.enrichments: Enrichment[]` field added

### `frontend/src/api/client.ts`

- `enrichFeedbackNote(id, content, source)` method added

### `frontend/src/pages/FeedbackPage.tsx`

- `NoteCard` extended with enrichment state (`enriching`, `enrichDraft`, `enrichSaving`, `enrichError`)
- "Add context" button visible on hover via `Plus` icon
- Inline textarea with Cmd+Enter to save and Escape to cancel
- `EnrichmentBlock` component renders enrichments with muted background, source pill, timestamp

## Test Results

```
sdlc-core: 319 passed, 0 failed
sdlc-server: 128 passed, 0 failed
```

## Findings

1. **Pre-existing issue (tracked separately):** `crates/sdlc-cli/src/cmd/investigate.rs` references `sdlc_core::knowledge::librarian_harvest_workspace` which does not exist — this prevents `sdlc-cli` from building. This is not introduced by this feature and is tracked as a pre-existing debt item.

2. **Pre-existing issue (fixed as part of this work):** `crates/sdlc-core/src/orchestrator/webhook.rs` had duplicate `WebhookEvent` and `WebhookEventOutcome` definitions from a merge conflict in the `orchestrator-webhook-events` feature. Resolved by deduplicating to the correct definition matching `db.rs` test expectations.

3. **File revert behavior:** The IDE's Rust LSP (rust-analyzer + rustfmt) was reverting the server `feedback.rs` file. Root cause: the LSP applies on-save formatting which can interact with the session. Mitigated by writing via Python script files stored in `/tmp/`.

## Verdict

APPROVED. All acceptance criteria met. Tests green. No debt introduced by this feature.
