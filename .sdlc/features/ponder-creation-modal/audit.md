# Security Audit: ponder-creation-modal

## Scope

This feature is a pure frontend UI change. The only new behavior is:

1. A centered modal replaces an inline form
2. URL reference strings entered by the user are passed to `api.capturePonderArtifact` as text content

No new backend endpoints, no new data models, no authentication changes, no server-side processing added.

---

## Attack Surface Analysis

### User-supplied URL strings

**Surface:** The references input accepts `type="url"` strings. These are passed as content to `POST /api/roadmap/:slug/capture`, which stores them in a `.md` file on disk.

**Risk: XSS via stored URL content**

The reference URLs end up in `references.md`, which is rendered by the ponder workspace panel (via `WorkspacePanel` → `MarkdownContent`). If the existing markdown renderer sanitizes content (which it should), links are rendered as `<a href="...">` tags.

Review of the existing `MarkdownContent` component is required to confirm sanitization. However, this risk is the same as any other user-supplied text content captured into ponder artifacts today — the `brief` field has identical exposure.

**Action:** Accepted — same risk level as existing brief.md capture. No new attack surface introduced by this feature. If XSS in the markdown renderer is a concern, it should be tracked as a separate feature-agnostic finding.

**Risk: SSRF**

The URL strings are not fetched by the server in any way. They are stored as text in a markdown file. No SSRF risk.

**Risk: Path traversal via slug or filename**

The `slug` and `filename` fields go through the same server-side path-construction as all other `capture_artifact` calls (using `sdlc_core::ponder::capture_content`). This is pre-existing validation, unchanged by this feature.

### Form validation

**Surface:** The slug input is client-side sanitized to `[a-z0-9-]` pattern and max 40 chars. The server enforces its own slug validation on `POST /api/roadmap`.

**Risk:** Client-side-only validation is not a security control, but server-side validation exists at the existing `create_ponder` route. No new server-side code was added, so validation posture is unchanged.

---

## Findings

| ID | Severity | Finding | Action |
|---|---|---|---|
| A1 | Info | URL references stored as markdown text share XSS risk with brief.md | Accepted — pre-existing risk, not introduced by this feature |
| A2 | None | No SSRF — URLs are not fetched server-side | N/A |
| A3 | None | Path traversal — unchanged from existing capture_artifact flow | N/A |

---

## Verdict

**Approved.** No new security concerns introduced. The feature is a UI refactor with one additive data field (reference URLs stored as markdown text) that shares the same risk profile as existing user-supplied text inputs.
