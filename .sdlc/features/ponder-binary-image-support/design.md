# Design: ponder-binary-image-support

## Overview

Three coordinated changes implement binary image support for the ponder workspace:

1. **`sdlc-core/src/paths.rs`** — add `ponder_media_dir` path helper
2. **`sdlc-server/src/routes/roadmap.rs`** — add upload (POST) and serve (GET) handlers
3. **`frontend/src/components/ponder/WorkspacePanel.tsx`** — render image artifacts inline

The design follows the same pattern as `milestones.rs`'s `get_uat_run_artifact`:
atomic read with path-traversal guard, correct MIME, no manifest changes.

---

## Layer 1 — Core path helper (`paths.rs`)

```rust
/// `.sdlc/roadmap/<slug>/media/` — storage root for binary media files.
pub fn ponder_media_dir(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join("media")
}
```

No other core changes are needed.  The `media/` subdirectory is created on first
upload by the server route (using `tokio::fs::create_dir_all`).

---

## Layer 2 — Server routes (`roadmap.rs`)

### 2a — MIME helper (local, mirrors milestones.rs)

```rust
fn ponder_mime_for_ext(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png"           => Some("image/png"),
        "jpg" | "jpeg"  => Some("image/jpeg"),
        "gif"           => Some("image/gif"),
        "webp"          => Some("image/webp"),
        _               => None,      // not an allowed image type
    }
}
```

Returning `None` means the file type is rejected.

### 2b — Upload handler

```
POST /api/roadmap/:slug/media
Content-Type: multipart/form-data  (field name: "file")
```

```rust
pub async fn upload_ponder_media(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError>
```

Algorithm:
1. Iterate multipart fields; find the field named `"file"`.
2. Extract `filename` from the `Content-Disposition` header.
3. **Path-traversal guard**: reject if filename contains `/`, `\`, or `..`.
4. **Type guard**: call `ponder_mime_for_ext(filename)` — return HTTP 400 if None.
5. Collect the field bytes with a 10 MB cap; return HTTP 413 if exceeded.
6. `tokio::fs::create_dir_all(ponder_media_dir(&root, &slug))`.
7. Atomic write: write to a `.tmp` file, then `tokio::fs::rename` into place
   (mirrors the pattern in `sdlc-core::io::write_file_atomic`).
8. Emit `SseMessage::PonderUpdated { slug: slug.clone() }` via `app.emit_sse`.
9. Return `{ "slug", "filename", "url": "/api/roadmap/{slug}/media/{filename}" }`.

### 2c — Serve handler

```
GET /api/roadmap/:slug/media/:filename
```

```rust
pub async fn serve_ponder_media(
    State(app): State<AppState>,
    Path((slug, filename)): Path<(String, String)>,
) -> Result<Response, AppError>
```

Algorithm:
1. **Path-traversal guard**: same check as upload.
2. `ponder_mime_for_ext(filename)` — return HTTP 400 if None (not an image type).
3. `tokio::fs::read(ponder_media_dir(&root, &slug).join(&filename))` — HTTP 404 on `Err`.
4. Build `Response` with `Content-Type` and binary body (same pattern as
   `get_uat_run_artifact` in milestones.rs).

### 2d — Route registration (`lib.rs`)

```rust
.route(
    "/api/roadmap/{slug}/media",
    post(routes::roadmap::upload_ponder_media),
)
.route(
    "/api/roadmap/{slug}/media/{filename}",
    get(routes::roadmap::serve_ponder_media),
)
```

These two lines slot in alongside the existing `/api/roadmap` routes.

### 2e — Cargo dependency

`axum-multipart` (via `axum::extract::Multipart`) is already available in Axum
0.8+.  Check `Cargo.toml` for the `axum` version; if the `multipart` feature is
not enabled, add `features = ["multipart"]` to the `axum` dependency in
`crates/sdlc-server/Cargo.toml`.

---

## Layer 3 — Frontend rendering (`WorkspacePanel.tsx`)

### 3a — Image detection helper

```ts
const IMAGE_EXTS = new Set(['.png', '.jpg', '.jpeg', '.gif', '.webp'])

function isImageArtifact(filename: string): boolean {
  const dot = filename.lastIndexOf('.')
  if (dot === -1) return false
  return IMAGE_EXTS.has(filename.slice(dot).toLowerCase())
}
```

### 3b — Artifact rendering

In the section of `WorkspacePanel` that renders artifact files, branch on type:

```tsx
{artifact.filename && isImageArtifact(artifact.filename) ? (
  <a
    href={`/api/roadmap/${slug}/media/${artifact.filename}`}
    target="_blank"
    rel="noopener noreferrer"
    className="block"
  >
    <img
      src={`/api/roadmap/${slug}/media/${artifact.filename}`}
      alt={artifact.filename}
      className="max-h-80 w-auto object-contain rounded border border-border"
    />
  </a>
) : (
  // existing Markdown content block
  <MarkdownBlock content={artifact.content} />
)}
```

- Max height 320 px (`max-h-80` in Tailwind) — image scales down, never overflows.
- Clicking the image opens the full-resolution version in a new tab (simple,
  no lightbox dependency required for v1).
- Non-image artifacts render unchanged.

### 3c — No new API calls needed

The images are served directly from `/api/roadmap/:slug/media/:filename` — the
browser fetches them as `<img src=...>` without any additional data loading.  The
existing `list_artifacts` call already returns filenames; the frontend switches
on extension.

---

## Sequence Diagram

```
Agent                    Server                   Filesystem
  |                        |                          |
  |-- POST /api/roadmap/   |                          |
  |   slug/media           |                          |
  |   (multipart PNG)      |                          |
  |                        |-- validate filename ---  |
  |                        |-- check ext/MIME type -- |
  |                        |-- size cap (10 MB) ----- |
  |                        |-- create_dir_all ------->|
  |                        |-- write .tmp file ------>|
  |                        |-- rename to final ------->|
  |                        |-- emit SSE PonderUpdated  |
  |<-- 200 { url } --------|                          |
  |                        |                          |
Browser                  Server                   Filesystem
  |                        |                          |
  |-- <img src=            |                          |
  |   /api/roadmap/        |                          |
  |   slug/media/file.png> |                          |
  |                        |-- validate filename ---  |
  |                        |-- check ext/MIME type -- |
  |                        |-- read bytes ----------->|
  |<-- 200 image/png ------|                          |
```

---

## Error Handling Summary

| Condition | HTTP | Body |
|---|---|---|
| Filename with `/`, `\`, or `..` | 400 | `"invalid filename: must not contain path separators or '..'"`  |
| Non-image extension | 400 | `"unsupported file type: only PNG, JPEG, GIF, WebP are allowed"` |
| File > 10 MB | 413 | `"file too large: maximum 10 MB"` |
| File not found (serve) | 404 | `"media file not found"` |

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/paths.rs` | Add `ponder_media_dir` helper + test |
| `crates/sdlc-server/Cargo.toml` | Ensure `axum` has `multipart` feature |
| `crates/sdlc-server/src/routes/roadmap.rs` | Add `ponder_mime_for_ext`, `upload_ponder_media`, `serve_ponder_media` + unit tests |
| `crates/sdlc-server/src/lib.rs` | Register two new routes |
| `frontend/src/components/ponder/WorkspacePanel.tsx` | `isImageArtifact` helper + conditional `<img>` render |

---

## Testing Strategy

### Rust unit tests (in `roadmap.rs`)

- `mime_for_ext_png` / `jpeg` / `gif` / `webp` → correct MIME strings
- `mime_for_ext_rejected` (`.exe`, `.txt`, no extension) → `None`
- `path_traversal_rejected` — filenames with `/`, `\`, `..`
- `ponder_media_dir_path` — path helper returns expected suffix

### Rust integration test (optional, for upload round-trip)

Use `axum::test` with a `TestClient` to POST a minimal PNG and assert the 200
response body and that the file exists on disk.  Follow the pattern of existing
`routes` integration tests.

### Manual verification

1. `curl -F "file=@screenshot.png" http://localhost:7777/api/roadmap/test-slug/media`
2. Open the returned URL in a browser — image renders.
3. Open PonderPage in the browser, navigate to a workspace with an uploaded image —
   thumbnail appears under the artifact list.
