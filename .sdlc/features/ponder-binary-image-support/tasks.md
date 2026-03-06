# Tasks: ponder-binary-image-support

## T1 — Add `ponder_media_dir` path helper to `paths.rs`

**File:** `crates/sdlc-core/src/paths.rs`

Add the following function after `ponder_sessions_dir`:

```rust
pub fn ponder_media_dir(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join("media")
}
```

Add a unit test in the existing `path_helpers` test or a new
`ponder_media_dir_path` test:

```rust
#[test]
fn ponder_media_dir_path() {
    let root = Path::new("/tmp/proj");
    assert_eq!(
        ponder_media_dir(root, "my-idea"),
        PathBuf::from("/tmp/proj/.sdlc/roadmap/my-idea/media")
    );
}
```

---

## T2 — Enable `multipart` feature in `axum` dependency

**File:** `crates/sdlc-server/Cargo.toml`

Locate the `axum` dependency entry. If `features` does not include `"multipart"`,
add it:

```toml
axum = { version = "...", features = ["multipart", ...] }
```

If it is already listed, this task is a no-op — confirm and mark complete.

---

## T3 — Implement `ponder_mime_for_ext` helper in `roadmap.rs`

**File:** `crates/sdlc-server/src/routes/roadmap.rs`

Add a private MIME helper that returns `Some` for allowed image types and `None`
for everything else (rejected):

```rust
fn ponder_mime_for_ext(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png"           => Some("image/png"),
        "jpg" | "jpeg"  => Some("image/jpeg"),
        "gif"           => Some("image/gif"),
        "webp"          => Some("image/webp"),
        _               => None,
    }
}
```

Add unit tests in the existing `#[cfg(test)]` block covering PNG, JPEG, GIF,
WebP, a rejected extension (`.exe`), and no extension.

---

## T4 — Implement `upload_ponder_media` handler

**File:** `crates/sdlc-server/src/routes/roadmap.rs`

```rust
pub async fn upload_ponder_media(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, AppError>
```

Implementation steps (in order):
1. Iterate `multipart.next_field()` until the field named `"file"` is found.
2. Extract the filename from the field; return HTTP 400 if absent.
3. Path-traversal guard: reject filename containing `/`, `\`, or `..` → HTTP 400.
4. Call `ponder_mime_for_ext(&filename)`; return HTTP 400 with
   `"unsupported file type: only PNG, JPEG, GIF, WebP are allowed"` if `None`.
5. Accumulate bytes with a 10 MB cap; return HTTP 413 if exceeded.
6. `tokio::fs::create_dir_all(sdlc_core::paths::ponder_media_dir(&root, &slug))`.
7. Write to a `.tmp` path and atomically rename to the final path.
8. Emit `app.emit_sse(SseMessage::PonderUpdated { slug: slug.clone() })` (or the
   equivalent SSE helper already used in other roadmap routes).
9. Return `Json(serde_json::json!({ "slug": slug, "filename": filename, "url": ... }))`.

---

## T5 — Implement `serve_ponder_media` handler

**File:** `crates/sdlc-server/src/routes/roadmap.rs`

```rust
pub async fn serve_ponder_media(
    State(app): State<AppState>,
    Path((slug, filename)): Path<(String, String)>,
) -> Result<Response, AppError>
```

Implementation steps:
1. Path-traversal guard (same check as T4) → HTTP 400.
2. `ponder_mime_for_ext(&filename)` → HTTP 400 if `None`.
3. Construct path: `sdlc_core::paths::ponder_media_dir(&app.root, &slug).join(&filename)`.
4. `tokio::fs::read(&path).await` → HTTP 404 on error.
5. Build and return `Response` with correct `Content-Type` and binary body,
   following the pattern of `get_uat_run_artifact` in `milestones.rs`.

---

## T6 — Register routes in `lib.rs`

**File:** `crates/sdlc-server/src/lib.rs`

Add after the existing `/api/roadmap/{slug}` routes:

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

Verify order does not conflict with any existing wildcard routes.

---

## T7 — Frontend: image detection and `<img>` rendering in `WorkspacePanel`

**File:** `frontend/src/components/ponder/WorkspacePanel.tsx`

1. Add the helper function near the top of the component file:

```ts
const IMAGE_EXTS = new Set(['.png', '.jpg', '.jpeg', '.gif', '.webp'])

function isImageArtifact(filename: string): boolean {
  const dot = filename.lastIndexOf('.')
  if (dot === -1) return false
  return IMAGE_EXTS.has(filename.slice(dot).toLowerCase())
}
```

2. In the artifact rendering section, replace the unconditional Markdown render
   with a conditional:

```tsx
{isImageArtifact(artifact.filename) ? (
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
  <MarkdownBlock content={artifact.content} />
)}
```

Adjust surrounding JSX as needed to match the actual component structure.

---

## T8 — Run tests and verify build

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All tests must pass.  Address any clippy warnings introduced by the new code.
