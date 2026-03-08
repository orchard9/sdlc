# Security Audit: Iterate Button on FeatureDetail

## Scope

Frontend-only change: a button in `FeatureDetail.tsx` that calls the existing `POST /api/roadmap` endpoint and a new pure utility function (`nextIterationSlug`).

## Findings

### A1: Input Validation — LOW RISK, ACCEPTED
The `nextIterationSlug` utility generates slugs from existing feature slugs (already validated by the backend). The generated slug is passed to `api.createPonderEntry` which validates on the server side. No user-provided free-text input reaches the slug.

### A2: API Surface — NO NEW SURFACE
Uses the existing `POST /api/roadmap` endpoint. No new backend routes, no new authentication paths, no new data exposure.

### A3: XSS — NOT APPLICABLE
The feature title is rendered via React's JSX which auto-escapes. The slug is used in a URL path (`/ponder/${newSlug}`), navigated via `react-router-dom`'s `navigate()` which does not interpret HTML.

### A4: Regex DoS — LOW RISK, ACCEPTED
The `escapeRegex` function and the version pattern regex are both simple and bounded. Input is a slug (short string, typically <60 chars). No catastrophic backtracking possible.

### A5: Data Leakage — NOT APPLICABLE
No sensitive data is exposed. The feature title and slug are already visible in the UI.

## Verdict

No security concerns. This is a minimal frontend addition using existing, validated APIs.
