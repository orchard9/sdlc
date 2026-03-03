# Security Audit: Wire up Thread Detail Actions

## Scope

New endpoints: `PATCH /api/threads/:id`, `POST /api/threads/:id/promote`
Modified frontend: `ThreadDetailPane.tsx`, `ThreadsPage.tsx`, `client.ts`

## Findings

### A1 — No authorization on delete/patch/promote (ACCEPT)

All thread mutation endpoints (`DELETE`, `PATCH`, `POST /promote`) are unauthenticated when accessed locally. This matches the existing posture of the entire API — all endpoints are unauthenticated locally and gated by tunnel token when accessed remotely (via `auth.rs` middleware). No regression introduced.

**Action:** Accept — consistent with project security model.

### A2 — Status field is a free-form string (ACCEPT)

`PATCH /api/threads/:id` accepts any `status` string. An adversary could set `status` to an arbitrary value. However:
- The frontend only sets `"synthesized"` via the Synthesize button
- The `StatusBadge` component has an exhaustive conditional — unknown statuses fall through to the "→ ponder" badge
- No server-side logic branches on status value in a security-relevant way

**Action:** Accept — no OWASP risk. Can add an allowlist in a follow-up if status values grow.

### A3 — Ponder slug derived from thread title (ACCEPT)

The `promote_thread` handler derives a ponder slug from the thread title. The derivation strips all non-alphanumeric characters and collapses dashes, so path traversal is not possible. The resulting slug is validated by `paths::validate_slug` inside `PonderEntry::create`.

**Action:** Accept — path traversal mitigated.

### A4 — No CSRF protection (ACCEPT)

Same-origin requests from the embedded SPA don't require CSRF tokens. The tunnel auth middleware provides the external access gate. No change in posture.

**Action:** Accept — existing model.

## Verdict

**APPROVED** — No new security risks introduced. All findings are consistent with the existing security model and are accepted with documented rationale.
