# Security Audit: uat-artifacts-ui

## Scope

Frontend-only changes. No Rust, no server-side code, no new API endpoints. This audit covers:

- `frontend/src/api/client.ts` — `uatArtifactUrl` URL builder
- `frontend/src/components/milestones/UatHistoryPanel.tsx` — filmstrip + `ScreenshotLightbox`
- `frontend/src/components/dashboard/MilestoneDigestRow.tsx` — hero thumbnail + `getLatestMilestoneUatRun` fetch

The binary artifact serving endpoint (`GET /api/milestones/{slug}/uat-runs/{id}/artifacts/{filename}`) is implemented by `uat-artifacts-storage` and covered in that feature's audit.

---

## Findings

### F-1: URL injection via `uatArtifactUrl` — mitigated

**Risk**: If `milestoneSlug`, `runId`, or `filename` contained path traversal sequences (`../`, `%2F`), a crafted URL could potentially target unintended server paths.

**Mitigation**: All three parameters are wrapped in `encodeURIComponent()` before interpolation. This encodes `/`, `..`, `%`, and other special characters. The resulting URLs are safe to use as `<img src>` attributes.

**Status: Mitigated — no action required**

### F-2: Reflected `src` in `<img>` tags — low risk

**Risk**: `<img src={...}>` attributes are populated from server-returned data (`screenshots: string[]`). If a malicious filename were injected into `UatRun.screenshots`, the browser would attempt to fetch it.

**Analysis**: `<img>` elements do not execute JavaScript. The `src` attribute triggers a GET request at most. Browsers sandbox image loads — no script execution occurs even for crafted filenames. The server-side endpoint controls what is actually served; the UI does not execute any content from the file, only displays it as an image. Cross-origin issues do not apply because all requests are same-origin.

**Status: Acceptable — no action required**

### F-3: `createPortal` DOM injection — not applicable

**Risk**: `createPortal(…, document.body)` appends elements to the document body. If content is user-controlled, this could theoretically inject HTML.

**Analysis**: All content in `ScreenshotLightbox` is either static strings (button labels, ARIA labels) or `<img src>` attributes from server data (covered by F-2). No `dangerouslySetInnerHTML` is used. No user-supplied strings are rendered as HTML.

**Status: Not applicable**

### F-4: One-time fetch in `MilestoneDigestRow` — no polling risk

**Risk**: A `useEffect` that fires repeatedly could create denial-of-service conditions against the API.

**Analysis**: The effect has `[milestone.slug]` as its dependency. It fires exactly once on mount, and again only if `milestone.slug` changes (which requires a route change). No interval or recursive scheduling is used.

**Status: Pass — no risk**

### F-5: No new authentication or authorization surface

This feature makes no changes to auth, token handling, or server-side access control. It reads from existing endpoints that are already gated by the sdlc-server auth middleware.

**Status: Pass**

---

## Summary

This is a frontend display feature with no new attack surface of note. URL encoding mitigates the only meaningful injection vector. No security findings require remediation.

**Verdict: Approved**
