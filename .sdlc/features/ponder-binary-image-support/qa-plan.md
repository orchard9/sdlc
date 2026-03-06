# QA Plan: ponder-binary-image-support

## Scope

Verify that binary image files can be uploaded to a ponder workspace, served
back with correct MIME types, and rendered inline in the browser workspace panel.
Confirm all security guards (path traversal, type restriction, size limit) are
enforced.

---

## TC-1 — Upload a valid PNG and receive the correct response

**Type:** Integration (Rust unit test or curl)

**Steps:**
1. Create a ponder entry with a known slug (e.g. `test-binary-upload`).
2. POST a minimal valid PNG file to `/api/roadmap/test-binary-upload/media`
   with `Content-Type: multipart/form-data`, field name `file`.
3. Assert HTTP 200.
4. Assert response JSON contains `slug`, `filename`, and `url` fields.
5. Assert `url` equals `/api/roadmap/test-binary-upload/media/<filename>`.

**Pass criteria:** 200 response; JSON body correct.

---

## TC-2 — Serve the uploaded file with correct MIME type

**Type:** Integration

**Steps:**
1. Upload a PNG as in TC-1.
2. GET the URL returned in TC-1.
3. Assert HTTP 200.
4. Assert `Content-Type` header is `image/png`.
5. Assert response body bytes equal the bytes that were uploaded.

**Pass criteria:** 200; `Content-Type: image/png`; byte-for-byte match.

---

## TC-3 — JPEG and WebP uploads

**Type:** Integration

**Steps:**
1. Upload a minimal JPEG file — assert HTTP 200 and `Content-Type: image/jpeg`
   when served.
2. Upload a minimal WebP file — assert HTTP 200 and `Content-Type: image/webp`
   when served.

**Pass criteria:** Both upload/serve round-trips succeed with correct MIME types.

---

## TC-4 — Reject non-image file type

**Type:** Unit / Integration

**Steps:**
1. POST a file with filename `malware.exe` to `/api/roadmap/<slug>/media`.
2. Assert HTTP 400.
3. Assert response body contains a descriptive error message referencing allowed
   image types.

**Pass criteria:** HTTP 400; no file written to disk.

---

## TC-5 — Reject file exceeding 10 MB

**Type:** Integration

**Steps:**
1. POST a file named `big.png` with 10 MB + 1 byte of content.
2. Assert HTTP 413.

**Pass criteria:** HTTP 413; no file written to disk.

---

## TC-6 — Path traversal protection (upload)

**Type:** Unit / Integration

**Steps:**
1. POST a file with filename `../../../etc/passwd` to the upload endpoint.
2. Assert HTTP 400.
3. Repeat with `subdir/evil.png`.
4. Repeat with `..\\etc\\passwd` (backslash).

**Pass criteria:** All three attempts return HTTP 400; no file written to disk.

---

## TC-7 — Path traversal protection (serve)

**Type:** Unit / Integration

**Steps:**
1. GET `/api/roadmap/<slug>/media/../manifest.yaml`.
2. Assert HTTP 400 (not 200, and not leaking file contents).

**Pass criteria:** HTTP 400.

---

## TC-8 — Serve a non-existent file returns 404

**Type:** Integration

**Steps:**
1. GET `/api/roadmap/<slug>/media/does-not-exist.png` (no prior upload).
2. Assert HTTP 404.

**Pass criteria:** HTTP 404.

---

## TC-9 — `ponder_media_dir` path helper returns correct path

**Type:** Rust unit test (in `paths.rs`)

**Steps:**
1. Call `ponder_media_dir(Path::new("/tmp/proj"), "my-idea")`.
2. Assert result equals `/tmp/proj/.sdlc/roadmap/my-idea/media`.

**Pass criteria:** Exact path match.

---

## TC-10 — MIME helper rejects unknown extensions

**Type:** Rust unit test (in `roadmap.rs`)

**Steps:**
1. Call `ponder_mime_for_ext("malware.exe")` → assert `None`.
2. Call `ponder_mime_for_ext("notes.txt")` → assert `None`.
3. Call `ponder_mime_for_ext("noextension")` → assert `None`.
4. Call `ponder_mime_for_ext("photo.PNG")` → assert `Some("image/png")`
   (extension matching is case-insensitive).

**Pass criteria:** All assertions pass.

---

## TC-11 — Frontend renders `<img>` for image artifacts

**Type:** Manual browser verification

**Steps:**
1. Build and run the dev server: `cargo run -p sdlc-server`.
2. Open the ponder workspace in the browser.
3. Select a ponder entry that has at least one uploaded image artifact.
4. Observe the workspace artifact list.
5. Confirm the image artifact renders as a `<img>` thumbnail (max-height 320 px).
6. Click the thumbnail; confirm the full image opens in a new tab.
7. Confirm non-image artifacts (Markdown `.md` files) still render as text.

**Pass criteria:** Image renders inline; click opens new tab; Markdown artifacts unchanged.

---

## TC-12 — SSE event emitted on successful upload

**Type:** Manual / Integration

**Steps:**
1. Open the ponder page in a browser with DevTools → Network → EventStream visible.
2. Upload a valid PNG via the upload endpoint.
3. Observe the SSE stream for a `PonderUpdated` event containing the correct slug.

**Pass criteria:** `PonderUpdated` event received within 2 seconds of upload.

---

## TC-13 — Cargo tests pass

**Type:** Automated

**Command:**
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

**Pass criteria:** Zero test failures; zero clippy warnings.

---

## Pass Criteria Summary

| TC | Description | Type | Must Pass |
|---|---|---|---|
| TC-1 | Upload PNG → 200 + correct JSON | Integration | Yes |
| TC-2 | Serve PNG → 200 + correct MIME + bytes | Integration | Yes |
| TC-3 | JPEG + WebP round-trips | Integration | Yes |
| TC-4 | Reject non-image extension → 400 | Unit/Integration | Yes |
| TC-5 | Reject >10 MB → 413 | Integration | Yes |
| TC-6 | Path traversal on upload → 400 | Unit/Integration | Yes |
| TC-7 | Path traversal on serve → 400 | Unit/Integration | Yes |
| TC-8 | 404 for missing file | Integration | Yes |
| TC-9 | `ponder_media_dir` path | Unit | Yes |
| TC-10 | MIME helper edge cases | Unit | Yes |
| TC-11 | Browser renders `<img>` for image artifacts | Manual | Yes |
| TC-12 | SSE event emitted on upload | Manual/Integration | Yes |
| TC-13 | `cargo test --all` + `clippy` green | Automated | Yes |
