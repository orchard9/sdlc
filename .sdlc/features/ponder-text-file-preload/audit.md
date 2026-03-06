# Security Audit: ponder-text-file-preload

## Scope

This audit covers the changes in `frontend/src/components/ponder/NewIdeaModal.tsx`, `frontend/src/lib/utils.ts`, and `frontend/src/components/ponder/WorkspacePanel.tsx`. No Rust or server code was changed.

## Attack Surface Analysis

### 1. File Content Handling — Client-Side Read

**Surface:** `file.text()` reads a user-selected local file and sends its content via `POST /api/roadmap/<slug>/capture`.

**Risk assessment:**
- The file is selected by the user from their own local filesystem via browser file picker or drag-and-drop.
- No path traversal is possible — the browser `File` API never exposes the file system path to JavaScript; only the `File` object itself is provided.
- `file.text()` calls the browser's built-in UTF-8 decoder. Malformed UTF-8 will produce replacement characters (`\uFFFD`), not errors or crashes.
- The content is sent as a JSON string in the request body. The server receives it as a `String` — no binary execution path exists.

**Verdict:** No additional risk beyond what the user could paste manually into a text field.

### 2. File Extension Filtering

**Surface:** `isAccepted(file)` uses a client-side allowlist of extensions.

**Risk assessment:**
- Client-side filtering is enforced only in the UI. A determined user could bypass it via the browser console and POST arbitrary content directly to `/api/roadmap/<slug>/capture`.
- However, that endpoint already accepted arbitrary filename + content pairs before this change. The allowlist is a UX guardrail, not a security boundary. The server does not need to enforce extension rules because the endpoint stores files as agent scrapbook artifacts — the content is plain text processed by an LLM agent, not executed.
- Binary content sent through the bypass path would be stored as garbled UTF-8, not executed.

**Verdict:** Acceptable. No regression from pre-existing surface.

### 3. Filename in API Payload

**Surface:** `file.name` is sent as the `filename` field in the capture payload.

**Risk assessment:**
- File names can contain arbitrary characters. The server routes through `capture_content` in `ponder.rs` which delegates to `workspace::write_artifact`.
- Code inspection confirms `workspace::write_artifact` already validates filenames: it rejects `../escape`, `sub/dir.md`, and empty strings (verified in `crates/sdlc-core/src/workspace.rs` lines 424-426 and test at line 666 in `ponder.rs`). An error is returned and the HTTP handler propagates a 4xx.
- Path traversal is fully mitigated server-side.

**Verdict:** Protected. No action needed.

### 4. Large File Denial-of-Service

**Surface:** Files up to any size can be attached and sent to the server.

**Risk assessment:**
- `file.text()` loads the entire file into memory in the browser. For very large files this could exhaust browser memory.
- The UI shows a warning for files > 500 KB but does not block them.
- The server does not enforce a body size limit specific to this endpoint; it relies on whatever the HTTP framework defaults are (Axum/hyper default: 2 MB body limit in most configurations).
- Risk is self-inflicted by the authenticated user. No unauthenticated path.

**Verdict:** Acceptable risk for an internal developer tool. No change needed in this feature.

### 5. XSS — File Content in Scrapbook

**Surface:** Captured file content is later rendered in `WorkspacePanel.tsx` via `ArtifactContent`.

**Risk assessment:**
- `ArtifactContent` renders Markdown. If the Markdown renderer does not sanitize HTML, a `.html` or `.md` file containing `<script>` tags could execute JavaScript when the artifact is displayed.
- This is a pre-existing concern for all scrapbook artifacts, not specific to this feature. References and agent-generated content could already inject HTML/script via the same render path.
- **Action:** Track for Markdown sanitization audit. No change to this feature.

**Verdict:** Pre-existing surface. Not introduced by this feature.

## Pre-existing Findings Logged

| Finding | Severity | Action |
|---------|----------|--------|
| Server-side filename: path traversal concern reviewed | N/A | Already mitigated; `write_artifact` rejects traversal paths |
| Markdown renderer may allow HTML injection in scrapbook view | Low | Pre-existing; tracked for future sanitization audit |

## Summary

No new security surfaces introduced by this feature. The one potential concern (path traversal via filename) was already handled by existing server-side validation. Markdown HTML injection is a pre-existing concern across all scrapbook content and is not introduced by this feature.
