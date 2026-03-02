# QA Plan: Edit Feedback Notes Inline

## Automated tests (Rust)

### sdlc-core unit tests (`crates/sdlc-core/src/feedback.rs`)

| ID | Test | Expected |
|---|---|---|
| U1 | `update_existing_note` — add a note, call `update` with new content | Returns `Some(note)` with updated `content` and `updated_at` set |
| U2 | `update_missing_note_returns_none` — call `update` with non-existent ID | Returns `Ok(None)` |
| U3 | `update_does_not_affect_other_notes` — add F1 and F2, update F1 | F2 unchanged; F1 content and `updated_at` updated |
| U4 | `existing_notes_without_updated_at_deserialise` — write raw YAML without `updated_at`, call `list` | Succeeds; `updated_at` is `None` on loaded notes |
| U5 | `id_counter_unaffected_by_update` — add, update, add again | Third note gets F3 (not reset) |

### sdlc-server route tests (`crates/sdlc-server/src/routes/feedback.rs`)

| ID | Test | Expected |
|---|---|---|
| R1 | `update_existing_note_returns_200` — add a note, PATCH with new content | 200 with updated note JSON including `updated_at` |
| R2 | `update_missing_note_returns_404` — PATCH `/api/feedback/F99` | 404 |
| R3 | `update_with_empty_content_returns_400` — PATCH with `{ "content": "" }` | 400 |
| R4 | `update_with_whitespace_only_returns_400` — PATCH with `{ "content": "   " }` | 400 |

Run with: `SDLC_NO_NPM=1 cargo test --all`

## TypeScript type check

```bash
cd frontend && npx tsc --noEmit
```

Must pass with zero errors.

## Clippy

```bash
cargo clippy --all -- -D warnings
```

Must pass clean.

## Manual smoke test (browser)

Requires `sdlc ui` running.

| ID | Steps | Expected |
|---|---|---|
| M1 | Open `/feedback`, add a note. Double-click the note body. | Textarea appears pre-filled with the note's text. |
| M2 | In edit mode, change text, press Cmd+Enter. | Textarea closes; updated text renders; metadata line shows "edited". |
| M3 | Open edit mode, press Escape. | Textarea closes; original text unchanged. |
| M4 | Open edit mode, clear all text, click Save. | Save button is disabled; no API call fired. |
| M5 | Hover over a note. | Pencil icon appears; clicking it opens edit mode. |
| M6 | Open edit mode on note A, then double-click note B. | Note A closes without saving; note B opens in edit mode. |
| M7 | Edit a note, save, refresh page. | Updated content persists after reload. |

## Regression checks

- Add, delete, and "Submit to Ponder" flows still work after the changes.
- Existing `feedback.yaml` files without `updated_at` load without error.
