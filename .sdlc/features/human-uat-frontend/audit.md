# Security Audit: Human UAT Frontend — Submission Modal and Secondary Buttons

## Scope

This feature adds a React modal component and two API POST calls. The security surface is purely frontend — no new server-side routes, no authentication changes, no new data storage. Backend endpoints (`/api/milestones/{slug}/uat/human` and `/api/features/{slug}/qa/human`) are implemented in the companion `human-uat-backend` feature and are not in scope here.

## Findings

### F1: User-Supplied Input — Notes Textarea

**Verdict: ACCEPT** — No action required.

The `notes` field contents are passed to the backend as a JSON string. The React rendering pipeline (`value={notes}`) handles HTML escaping automatically. The content is never rendered as raw HTML — it is sent to the API and the backend is responsible for safe storage. No XSS surface.

### F2: No Authentication on Client-Side

**Verdict: ACCEPT** — By design.

The API calls use the same relative `fetch` pattern (`/api/...`) as every other call in the codebase. Authentication is enforced at the server layer (tunnel token middleware in `crates/sdlc-server/src/auth.rs`). The frontend does not add, manage, or expose tokens.

### F3: Overlay Click-to-Close

**Verdict: ACCEPT** — No information-disclosure risk.

Clicking outside the modal calls `onClose()`, which just hides the modal. No data is submitted. The `submitting` guard prevents accidental dismissal during in-flight requests.

### F4: Checklist Fetch — External Content

**Verdict: ACCEPT** — Minimal risk.

The checklist is fetched from `/api/artifacts/{slug}/qa_plan` or `/api/milestones/{slug}/acceptance-test` — both are internal endpoints serving project-owned Markdown. The content is rendered inside a `<pre>` element with `whitespace-pre-wrap`, not via `dangerouslySetInnerHTML`. No XSS risk from the checklist content.

### F5: Verdict and Notes — Tampering in Transit

**Verdict: ACCEPT** — Out of scope for frontend.

The JSON payload is sent over HTTPS (enforced by the tunnel/proxy layer). The backend is responsible for validating the `verdict` enum value. The frontend does client-side validation (verdict required, notes conditionally required) as UX improvement only, not as a security gate.

## Summary

This feature has no meaningful new security surface. All data flows are consistent with the existing codebase patterns. No findings require remediation.
