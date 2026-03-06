# Security Audit: Artifact and Dialogue Copy & Screenshot

## Surface

Client-side only. No backend changes. No new API routes, no new server-side data handling. The feature reads content from already-loaded React state and writes to the clipboard or downloads a file.

## Findings

### Clipboard API — content origin trust

**Severity: Informational**

`CopyButton` writes `artifact.content` or `message.content` to the clipboard. These values originate from the sdlc server (authenticated endpoints) and are already visible in the DOM. There is no new exposure: the user is copying content they can already read. No finding.

### html2canvas — captured element scope

**Severity: Informational**

`CaptureButton` calls `html2canvas(targetRef.current)`. The `targetRef` is a `useRef<HTMLDivElement>` scoped to the artifact content panel or dialogue bubble. It cannot capture outside its DOM subtree — html2canvas renders only the referenced element and its children. No cross-origin content can appear in the capture (the artifact content is sdlc-server-owned). No finding.

### html2canvas — `useCORS: true` option

**Severity: Low — Accepted**

`useCORS: true` allows html2canvas to request cross-origin images with a CORS header. This is only relevant if artifact content includes `<img>` tags referencing external URLs. Artifact markdown content is agent-generated and server-stored — external images are unlikely but not impossible (e.g. a markdown spec with an external image link).

**Risk:** An agent could write an artifact containing a cross-origin image URL. With `useCORS: true`, the browser will attempt a CORS preflight — if the image server allows CORS, the image is captured; if not, html2canvas renders a blank area. No data is exfiltrated: the capture goes to the user's own clipboard. **Accepted:** low impact, no user-controllable cross-origin fetch path.

### ClipboardItem write permission

**Severity: Informational**

`navigator.clipboard.write()` requires the `clipboard-write` permission, automatically granted by browsers when triggered by a direct user gesture. Our handler is synchronous from a click event, so the permission is always granted. If it is somehow denied, the fallback downloads the file instead. No finding.

### html2canvas dynamic import — supply chain

**Severity: Informational — Standard Practice**

`html2canvas@1.4.1` is a well-maintained open source library. Dynamic import does not change the supply chain risk vs. a static import — both resolve from `node_modules` at build time. The library does not make network requests at runtime.

## Verdict

No security findings that require action. One low-severity accepted finding (useCORS with external images). The feature surface is entirely client-side and introduces no new data flows to or from the server.
