# Spec: ponder-binary-image-support

## Summary

Enable ponder workspace participants (agents and users) to attach binary image
files to ponder sessions and scrapbook artifacts — and have those images rendered
inline in the workspace UI.  Today the workspace only handles text (Markdown
files); screenshots, diagrams, and other visual assets captured during ideation
sessions are silently lost or must be manually linked via external URLs.

## Problem Statement

The ponder workspace is the primary ideation surface for ideas before they become
features.  Agents and humans produce visual outputs during sessions — Playwright
screenshots, architecture diagrams, whiteboard snapshots, charts.  There is
currently no path to:

1. Upload a binary image file into a ponder workspace
2. Serve that file back to the browser through the API
3. Render the image inline in the workspace panel

This forces workarounds (external hosting, base64-encoded text blocks) that are
fragile and block the vision of rich, fully-captured ponder sessions.

## Goals

1. A multipart upload endpoint that accepts image files and writes them to the
   ponder workspace directory under `.sdlc/roadmap/<slug>/media/`.
2. A GET route that serves binary media files from that directory with the correct
   MIME type and path-traversal protection.
3. Frontend support: the WorkspacePanel (and any file-list component) renders `<img>` tags
   for image artifacts, fetching from the new serve route.

## Non-Goals

- Video or audio file support (out of scope; only images: PNG, JPEG, GIF, WebP)
- Per-user access control (inherits existing tunnel auth)
- Image resizing or transcoding
- Upload from the ponder dialogue chat input (future; this feature covers
  programmatic upload by agents and manual file upload in the workspace panel)

## Requirements

### R1 — Upload endpoint

```
POST /api/roadmap/:slug/media
Content-Type: multipart/form-data
  field: file  (the image binary)
```

- Accepts PNG, JPEG, GIF, WebP by MIME type or file extension.
- Rejects files whose extension or declared MIME type is not an allowed image type
  (HTTP 400 with descriptive error).
- Saves the file as `.sdlc/roadmap/<slug>/media/<filename>` using atomic write
  (via `sdlc-core::io`).
- Returns `{ "slug": "...", "filename": "...", "url": "/api/roadmap/:slug/media/:filename" }`.
- Max file size: 10 MB — exceeding this returns HTTP 413.
- Path traversal guard: filename must not contain `/`, `\`, or `..`.
- On success, emits an SSE `PonderUpdated { slug }` event so the UI refreshes.

### R2 — Serve endpoint

```
GET /api/roadmap/:slug/media/:filename
```

- Reads `.sdlc/roadmap/<slug>/media/<filename>` from disk.
- Returns the file with the correct `Content-Type` header (reuse
  `mime_for_filename` from `milestones.rs`, moved to a shared module or
  duplicated in `roadmap.rs`).
- Path traversal guard identical to R1.
- Returns HTTP 404 if the file does not exist.
- No auth check beyond the global tunnel middleware (consistent with other
  `/api/roadmap` routes).

### R3 — Core path helper

Add `ponder_media_dir(root, slug) -> PathBuf` to `crates/sdlc-core/src/paths.rs`
returning `.sdlc/roadmap/<slug>/media/`.

### R4 — Frontend image rendering

In `WorkspacePanel` (and related ponder artifact display components):
- When listing artifacts for a ponder entry, detect image files by extension
  (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`).
- Render a thumbnail `<img src="/api/roadmap/:slug/media/:filename">` with
  `object-fit: contain`, max height 320 px, and a click-to-expand lightbox
  (or a link to the full URL in a new tab as a simpler fallback).
- Non-image artifacts continue to render as before (Markdown content block).

### R5 — No new database / no manifest changes

Media files are stored as plain files in the `media/` subdirectory.  The
`PonderEntry` manifest is not modified.  The existing `list_artifacts` function
already scans the directory — it will pick up media files as artifacts; the
frontend uses the extension to decide how to render them.

## Data Model

```
.sdlc/roadmap/<slug>/
  manifest.yaml        (unchanged)
  team.yaml            (unchanged)
  sessions/            (unchanged)
  media/               (new)
    screenshot-01.png
    diagram.webp
  *.md                 (existing text artifacts)
```

## API Contract

### Upload

```
POST /api/roadmap/:slug/media
Content-Type: multipart/form-data; boundary=...

--boundary
Content-Disposition: form-data; name="file"; filename="screenshot.png"
Content-Type: image/png

<binary data>
--boundary--
```

Response 200:
```json
{
  "slug": "my-feature-idea",
  "filename": "screenshot.png",
  "url": "/api/roadmap/my-feature-idea/media/screenshot.png"
}
```

### Serve

```
GET /api/roadmap/my-feature-idea/media/screenshot.png
```

Response 200 with `Content-Type: image/png` and binary body.

## Acceptance Criteria

1. An agent or curl client can POST a PNG file to
   `/api/roadmap/<slug>/media` and receive a 200 response with the correct URL.
2. The served GET URL returns the exact bytes that were uploaded with
   `Content-Type: image/png`.
3. Uploading a `.exe` or other non-image file returns HTTP 400.
4. Uploading a file larger than 10 MB returns HTTP 413.
5. A filename containing `../` returns HTTP 400 (path traversal protection).
6. The WorkspacePanel renders an `<img>` element for the uploaded file when
   viewing the ponder workspace in the browser.
7. `SDLC_NO_NPM=1 cargo test --all` passes with new unit tests covering
   path helper, MIME detection, and path-traversal guard.
