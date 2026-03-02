# Design: backlog-server

## Architecture

This feature is a thin HTTP wrapper over the existing `sdlc_core::backlog::BacklogStore`. No new data model, no new business logic — the Rust layer is already complete. The design is purely about exposing it via Axum routes in `sdlc-server`.

### File Structure

```
crates/sdlc-server/
  src/
    routes/
      backlog.rs        ← NEW — four async handler functions
      mod.rs            ← add `pub mod backlog;`
    lib.rs              ← register four routes in build_router_from_state
  tests/
    integration.rs      ← add integration tests for all four routes
```

### Handler Design

All handlers follow the standard pattern established in `knowledge.rs`, `advisory.rs`, and other routes:
1. Accept `State<AppState>` to access `app.root`
2. Wrap the blocking `sdlc_core` call in `tokio::task::spawn_blocking`
3. Map the error with `AppError` and return `Json<serde_json::Value>`

The `BacklogItem` struct derives `Serialize`, so it serializes cleanly to JSON with `serde_json::to_value`. All optional fields (`description`, `evidence`, `source_feature`, `park_reason`, `promoted_to`) serialize as `null` when absent due to `#[serde(skip_serializing_if = "Option::is_none")]` in the core struct — these will be absent from JSON, not null. The client must handle absent fields.

### Route Registration

Routes are added to the top-level Axum router in `lib.rs` in the existing chain. No middleware changes needed — auth middleware wraps all routes globally. Order within the chain: add after knowledge routes, before investigations, following existing grouping conventions.

```
GET  /api/backlog               → routes::backlog::list_backlog
POST /api/backlog               → routes::backlog::create_backlog_item
POST /api/backlog/{id}/park     → routes::backlog::park_backlog_item
POST /api/backlog/{id}/promote  → routes::backlog::promote_backlog_item
```

### Status Parsing

The `?status=` query parameter is a string that maps to `BacklogStatus`. Parsing uses `match` rather than `FromStr` since `BacklogStatus` does not implement `FromStr`. Invalid values return `400 Bad Request`.

```rust
#[derive(serde::Deserialize, Default)]
pub struct ListBacklogQuery {
    pub status: Option<String>,
    pub source_feature: Option<String>,
}
```

Parse `status` string inside `spawn_blocking`:
```rust
let status_filter = params.status.as_deref().map(|s| match s {
    "open" => Ok(BacklogStatus::Open),
    "parked" => Ok(BacklogStatus::Parked),
    "promoted" => Ok(BacklogStatus::Promoted),
    other => Err(SdlcError::InvalidSlug(format!("unknown status: {other}"))),
}).transpose()?;
```

### Kind Parsing (POST body)

`BacklogKind` is already `Deserialize`, so `Json<CreateBacklogBody>` handles it directly. The Axum JSON extractor returns `422` on deserialization failure (unknown enum variant).

### Promote Handler — optional slug

The `promote` endpoint calls `BacklogStore::mark_promoted` which requires a `feature_slug` parameter. When `slug` is omitted from the request body, use an empty string `""` as a sentinel — but this is incorrect per the core API. Instead, use a synthetic placeholder if absent:

After review: `mark_promoted` stores the slug as `promoted_to`. If `slug` is omitted, we should still call `mark_promoted` with some value. Use the item's ID as the fallback (`promoted_to: item_id`). However, since `slug` is optional per spec, the handler will use `"<unknown>"` if absent. This is acceptable — the CLI promote flow always provides a slug.

Revised: if `slug` is None, pass `""` and let `promoted_to` be `Some("")`. Callers providing no slug accept that the promoted_to field will be blank. This mirrors the spec intent that `slug` is optional.

### Error Mapping

Existing `error.rs` already maps:
- `SdlcError::BacklogItemNotFound` → `404`
- `SdlcError::InvalidTransition` → `422`
- `SdlcError::InvalidSlug` → `400`

No `error.rs` changes needed.

### HTTP Status Codes

| Scenario | Status |
|---|---|
| List — success | 200 |
| Create — success | 201 |
| Park — success | 200 |
| Promote — success | 200 |
| Item not found | 404 |
| Empty park_reason | 422 |
| Park promoted item | 422 |
| Already promoted | 422 |
| Unknown status query param | 400 |
| Missing required field | 422 (Axum JSON extraction failure) |

### Integration Test Strategy

Tests use `TempDir` + `init_project` helper (already defined in `tests/integration.rs`). Each test builds a fresh router via `sdlc_server::build_router(dir.path().to_path_buf(), 0)` and sends requests via `tower::ServiceExt::oneshot`.

Tests reuse existing `get()` and `post_json()` helpers defined at the top of `tests/integration.rs`.

No mocking needed — the handlers call real `BacklogStore` functions on temp files.

## What This Feature Does NOT Do

- No SSE events — backlog mutations do not emit SSE. Adding SSE would require new `SseMessage` variants, which is a separate feature.
- No frontend UI — this is the server-side API only. The UI integration is a separate feature.
- No pagination — the backlog is expected to be short (< 200 items at any time).
- No `DELETE` endpoint — backlog items are never deleted, only parked or promoted.
- No `GET /api/backlog/:id` — not in scope per feature description. The list endpoint serves as the primary read path.
