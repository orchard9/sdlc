# Audit: Dashboard Four-Zone Layout

## Security Surface

This change is purely a frontend restructuring — no new API endpoints, no new data
fetches, no new auth paths. All data comes from the existing `useProjectState()`
hook via SSE, same as before.

## Findings

### A1 — No new data exposure
All data displayed in zone components was already displayed in the old Dashboard.
The refactor does not surface any new fields from ProjectState. ✓

### A2 — No new user inputs
AttentionZone contains the EscalationCard resolve form — this was already present
in Dashboard.tsx. No new inputs added. The form calls `api.resolveEscalation` which
routes through the existing auth-protected server endpoint. ✓

### A3 — Link targets
All `<Link to="...">` values use hardcoded relative paths (`/features/${f.slug}`,
`/milestones/${m.slug}`, `/setup`, `/secrets`). Slugs come from server-provided
state, not user input. No open-redirect risk. ✓

### A4 — Command text in CommandBlock
`/sdlc-run ${nextFeature.slug}` is rendered as code text only — no `eval`, no
`dangerouslySetInnerHTML`, no shell execution in the browser. CopyButton copies
the text to clipboard only. ✓

### A5 — CSP / XSS
No `innerHTML`, no `dangerouslySetInnerHTML`, no dynamic script loading added.
React renders all content as text nodes. ✓

## Verdict

No security issues. Change is a pure UI restructure with no new attack surface.
