# Design: Spike REST Routes

## Summary

This is a pure backend feature — three thin HTTP handlers over the existing
`sdlc_core::spikes` data layer. No new data structures are introduced at the
server layer. No UI is included.

## File Layout

```
crates/sdlc-server/src/routes/spikes.rs    (new)
crates/sdlc-server/src/routes/mod.rs       (add `pub mod spikes;`)
crates/sdlc-server/src/lib.rs              (register 3 routes)
```

## Route Handlers

### `list_spikes` — GET /api/spikes

```
State(app): State<AppState>
  → spawn_blocking { sdlc_core::spikes::list(&root) }
  → serialize each SpikeEntry as JSON object
  → return Json(array)
```

SpikeEntry fields exposed:
- `slug`, `title`, `verdict` (serialized as string or null), `date`,
  `the_question`, `ponder_slug`, `knowledge_slug`

### `get_spike` — GET /api/spikes/:slug

```
State(app): State<AppState>, Path(slug): Path<String>
  → spawn_blocking { sdlc_core::spikes::load(&root, &slug) }
  → returns (SpikeEntry, findings_markdown)
  → serialize entry fields + "findings" string field
  → return Json(object)
```

404 mapping: `sdlc_core::SdlcError::Io(e)` where `e.kind() == NotFound`
  → `AppError` returning HTTP 404.

### `promote_spike` — POST /api/spikes/:slug/promote

```
State(app): State<AppState>, Path(slug): Path<String>,
Json(body): Json<PromoteBody>
  → spawn_blocking {
      let (entry, _) = sdlc_core::spikes::load(&root, &slug)?;
      if entry.verdict != Some(SpikeVerdict::Adapt) {
          return Err(422 — only ADAPT spikes are promoted);
      }
      let ponder_slug = sdlc_core::spikes::promote_to_ponder(
          &root, &slug, body.ponder_slug.as_deref()
      )?;
      Ok(json!({ "ponder_slug": ponder_slug }))
    }
```

`PromoteBody`:
```rust
#[derive(serde::Deserialize)]
struct PromoteBody {
    #[serde(default)]
    ponder_slug: Option<String>,
}
```

## Error Handling

The server uses `AppError(anyhow::Error)` which renders as 500 by default.
For 404 and 422 we need explicit HTTP status codes.

Options:
1. Return `(StatusCode, Json<Value>)` tuple for error cases
2. Extend `AppError` to carry a status code

Pattern used by other routes: they return `AppError` and rely on `500`.
For `get_spike` 404 and `promote_spike` 422 we will return
`(StatusCode, Json<Value>)` directly — the handlers return
`Result<Json<Value>, (StatusCode, Json<Value>)>` for those two, and
convert `AppError` into the tuple form where needed.

Actually, simpler: return `Result<Json<Value>, AppError>` and check the error
type inside spawn_blocking, converting to a structured `AppError` that includes
status info. Looking at the existing `AppError`:

```rust
// crates/sdlc-server/src/error.rs
pub struct AppError(pub anyhow::Error);
impl IntoResponse for AppError { /* returns 500 */ }
```

Best fit: add a helper `not_found(msg)` and `unprocessable(msg)` that return
`(StatusCode, Json<Value>)` as the error type only for routes that need it.
Cleaner: use `axum::response::Response` as the error type so we can return
any status.

**Decision**: For `get_spike` and `promote_spike`, return
`Result<Json<Value>, axum::response::Response>` where the error path builds
an appropriate HTTP response with status + JSON body. This avoids modifying
`AppError` which is shared infrastructure.

## Route Registration (lib.rs)

```rust
.route("/api/spikes", get(routes::spikes::list_spikes))
.route("/api/spikes/{slug}", get(routes::spikes::get_spike))
.route(
    "/api/spikes/{slug}/promote",
    post(routes::spikes::promote_spike),
)
```

Placed after the roadmap/advisory routes group and before the auth middleware
layer — consistent with all other `/api/*` routes.

## Data Flow Diagram

```
HTTP GET /api/spikes
  → list_spikes handler
  → spawn_blocking
    → sdlc_core::spikes::list(root)
      → reads .sdlc/spikes/*/findings.md  (parse_findings)
      → reads .sdlc/spikes/*/state.yaml   (read_state)
      → side-effect: auto-files REJECT spikes into knowledge base
    → returns Vec<SpikeEntry>
  → serialize to JSON array
  → HTTP 200 + JSON

HTTP GET /api/spikes/:slug
  → get_spike handler
  → spawn_blocking
    → sdlc_core::spikes::load(root, slug)
      → validates slug
      → reads findings.md + state.yaml
    → returns (SpikeEntry, String)
  → serialize to JSON object with "findings" field
  → HTTP 200 + JSON
  → on NotFound: HTTP 404 + { "error": "spike '...' not found" }

HTTP POST /api/spikes/:slug/promote
  → promote_spike handler
  → spawn_blocking
    → sdlc_core::spikes::load(root, slug) to check verdict
    → verdict != Adapt → HTTP 422 + { "error": "only ADAPT spikes can be promoted" }
    → sdlc_core::spikes::promote_to_ponder(root, slug, override)
      → creates .sdlc/roadmap/<ponder_slug>/
      → writes spike-findings.md, open-questions.md artifacts
      → writes state.yaml (ponder_slug)
    → returns ponder_slug
  → HTTP 200 + { "ponder_slug": "..." }
```

## No Migration Required

All data is already on disk in `.sdlc/spikes/`. These routes are purely additive.
