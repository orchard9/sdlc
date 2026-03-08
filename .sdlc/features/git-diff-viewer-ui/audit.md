# Security Audit: git-diff-viewer-ui

## Scope

Frontend-only feature: `DiffViewer.tsx`, `DiffViewer.css`, and the `@git-diff-view/react` npm dependency.

## Findings

### SA-1: XSS via diff content rendering

- **Risk**: The `@git-diff-view/react` library renders diff content that could contain malicious HTML/JS if the file being diffed contains script tags or event handlers.
- **Assessment**: LOW — The library renders content as text nodes within its virtual DOM, not as raw HTML. The `DiffFile` parser treats all content as plain text. React's default escaping provides additional protection.
- **Action**: Accept — the library's text-based rendering is safe. No raw `dangerouslySetInnerHTML` is used.

### SA-2: API path injection

- **Risk**: The `filePath` prop is passed directly to `URLSearchParams({ path: filePath })` which could allow path traversal if a malicious path is provided.
- **Assessment**: LOW — This is a frontend component. The actual path validation must occur server-side in the `GET /api/git/diff` endpoint (part of the `git-diff-api` feature). The frontend cannot enforce server-side path restrictions.
- **Action**: Accept — server-side validation is the correct enforcement point and is the responsibility of the `git-diff-api` feature.

### SA-3: Third-party dependency (@git-diff-view/react v0.1.1)

- **Risk**: New npm dependency with transitive dependencies (highlight.js, lowlight, fast-diff, reactivity-store, use-sync-external-store).
- **Assessment**: LOW — All transitive deps are well-known, widely-used libraries. `highlight.js` and `lowlight` are industry-standard syntax highlighters. `npm audit` reports 0 vulnerabilities.
- **Action**: Accept — dependencies are reputable and vulnerability-free at time of installation.

### SA-4: No authentication bypass

- **Risk**: The component fetches from `/api/git/diff` — does this bypass auth?
- **Assessment**: NONE — The component uses relative URLs which go through the same Axum server with existing auth middleware. No auth bypass is possible from a frontend component.
- **Action**: Accept.

## Verdict

**APPROVED** — No security issues identified. The component is purely presentational with safe text rendering. Server-side path validation is correctly deferred to the backend API feature.
