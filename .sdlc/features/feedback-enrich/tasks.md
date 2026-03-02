# Tasks: Enrichments — Attach Research Context to Feedback Notes

## T1: Add `Enrichment` struct and `enrichments` field to `FeedbackNote`

Add `Enrichment { source, content, added_at }` struct and `enrichments: Vec<Enrichment>` field to `FeedbackNote` in `sdlc-core/src/feedback.rs` with `#[serde(default)]`.

## T2: Implement `feedback::enrich()` function

Add `pub fn enrich(root, id, source, content) -> Result<FeedbackNote>` to `sdlc-core/src/feedback.rs`. Returns `SdlcError::FeedbackNoteNotFound` if not found.

## T3: Update `to_markdown` to include enrichment blocks

Extend `to_markdown` to render enrichment blocks below each note in `> **Context** [source] -- _timestamp_` format.

## T4: Add `SdlcError::FeedbackNoteNotFound` and server error mapping

Add the variant to `sdlc-core/src/error.rs` and map it to HTTP 404 in `sdlc-server/src/error.rs`.

## T5: Add `enrich_note` handler and register route

Add `POST /api/feedback/{id}/enrich` handler in `sdlc-server/src/routes/feedback.rs`, extend `note_to_json` to include `enrichments`, and register the route in `lib.rs`.

## T6: Update frontend TypeScript and API client

Add `Enrichment` interface, update `FeedbackNote` type, and add `enrichFeedbackNote` API client method in `frontend/src/lib/types.ts` and `frontend/src/api/client.ts`.

## T7: Update FeedbackPage NoteCard UI

Add "Add context" button on hover, inline textarea for enrichment input, Cmd+Enter save, Escape cancel, and enrichment block rendering in `frontend/src/pages/FeedbackPage.tsx`.
