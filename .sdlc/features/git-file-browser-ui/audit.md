# Security Audit: File Browser Component

## Scope

Frontend-only React components: `useGitFiles` hook, `StatusBadge`, `GitFileBrowser`, and integration into `GitPage`.

## Findings

### 1. API Data Consumption (Low Risk)
**Finding:** The `useGitFiles` hook fetches from `/api/git/files` (a read-only GET endpoint) and renders the response data in the UI.
**Action:** Accepted. The data is rendered as text content via React's default escaping (no `dangerouslySetInnerHTML`). File paths are displayed in `<span>` elements, so XSS from malicious filenames is not possible. No user input is sent to the backend.

### 2. localStorage Usage (Low Risk)
**Finding:** View mode preference is stored in `localStorage` under key `git-file-browser-view`.
**Action:** Accepted. The stored value is read with a type assertion and defaults to `'flat'` if unrecognized. No sensitive data is stored. The try/catch wrapper handles environments where localStorage is unavailable.

### 3. URL Navigation (Low Risk)
**Finding:** File selection navigates to `/git/<filepath>` via `react-router-dom`'s `navigate()`.
**Action:** Accepted. The path is constructed from server-provided file paths, not user input. React Router handles URL encoding. The route is read-only — no mutations triggered by navigating to a file path.

### 4. No Authentication/Authorization Surface
This component displays workspace files already accessible to the authenticated user. No new auth decisions are introduced.

## Verdict

No security issues found. The component is a read-only UI consumer of an existing API endpoint with proper data rendering via React's built-in XSS protections.
