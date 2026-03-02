# Security Audit: Enrichments — Attach Research Context to Feedback Notes

## Scope

New attack surface introduced:
- `POST /api/feedback/:id/enrich` — new HTTP endpoint accepting user-controlled string input
- `EnrichBody { content: String, source: String }` — deserialized from JSON body

## Authentication & Authorization

The server is protected by the existing tunnel auth middleware (`crates/sdlc-server/src/auth.rs`). All `/api/*` routes require token/cookie authentication or local-only bypass. No new auth rules are needed for this endpoint — it inherits the same protection as the existing `/api/feedback` routes.

Finding: PASS — no auth gaps introduced.

## Input Validation

- `content` and `source` are stored as plain strings in YAML via serde. No HTML rendering, no SQL injection surface, no shell execution.
- `source` is user-controlled but is treated as metadata only (stored, serialized to JSON, never executed).
- No size limits on `content` or `source`. This is consistent with existing feedback note `content` handling — the application is a local-first developer tool, not a multi-tenant SaaS.
- No deserialization of untrusted types beyond `String`.

Finding: PASS — string storage only, consistent with existing behavior. Size limits are a future hardening opportunity, not a blocker.

## Persistence

Data is written to `.sdlc/feedback.yaml` via `sdlc-core::io::atomic_write` (write-to-temp, rename). The path is derived from the application root, not from user input.

Finding: PASS — no path traversal possible.

## Error Handling

`FeedbackNoteNotFound` returns HTTP 404 without leaking filesystem paths or internal state. The error message is "feedback note '{id}' not found" — the `id` is echoed back but this is the same `id` the caller provided, so no information leakage.

Finding: PASS.

## Backward Compatibility / Data Integrity

`#[serde(default)]` on `enrichments` ensures no corruption of existing data. Atomic writes prevent partial writes on crash.

Finding: PASS.

## Summary

| Finding | Severity | Status |
|---------|----------|--------|
| No auth gap | - | PASS |
| No injection surface | - | PASS |
| No path traversal | - | | PASS |
| No info leakage | - | PASS |
| Content size limits absent | LOW | ACCEPTED — consistent with existing behavior, local tool |

No blocking findings. Feature is cleared for merge.
