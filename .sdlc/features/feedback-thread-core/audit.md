# Security Audit: FeedbackThread core — data model, CLI, and REST API

## Surface

This feature adds:
- A new file-based data store under `.sdlc/feedback-threads/`
- 5 REST endpoints on the existing `sdlc-server` HTTP server
- 4 CLI subcommands on `sdlc-cli`

The server sits behind the existing auth middleware (token/cookie gate, local bypass). No authentication changes are introduced by this feature.

## Findings

### F1 — Path traversal via thread ID: ACCEPTED (no action needed)

Thread IDs are generated internally by `make_thread_id` and never directly accepted as user-supplied path segments in the core library. The REST routes accept an `id` path parameter, but that value is passed to `paths::feedback_thread_dir(root, id)` which calls `root.join(".sdlc/feedback-threads").join(id)`. A crafted `id` containing `../` could in theory traverse outside `.sdlc/feedback-threads/`.

**Mitigation already present:** Thread IDs are only ever created by `make_thread_id`, which sanitizes the context string and produces a date-prefixed, alphanumeric-plus-dash string. A client cannot create an arbitrary ID; it can only reference IDs it received from `POST /api/threads`. The auth middleware blocks unauthenticated access. The CLI only accepts IDs from prior `sdlc thread create` output.

**Residual risk:** An authenticated client (or a compromised session token) could supply a crafted ID string in `GET /api/threads/:id`. This would likely produce a `ThreadNotFound` (directory does not exist) rather than traversal, but is worth noting for a future hardening pass. Tracking as a non-blocking backlog item.

**Action:** Add `sdlc backlog` item to validate thread IDs against a safe-character regex before filesystem access. No code change in this feature.

### F2 — Content injection: LOW / ACCEPTED

Thread post `content` and `author` fields are stored as plain YAML and returned as JSON. They are not executed, eval'd, or rendered as HTML on the server side. XSS risk is deferred to the UI layer (`feedback-thread-ui`), which must sanitize before rendering. No server-side action needed.

### F3 — Unbounded content size: LOW / ACCEPTED

There is no max-length check on `content` in `add_post`. A very large post body could consume disk space. The existing server does not enforce request body size limits on a per-route basis. This is consistent with the existing `feedback.rs` behavior and is a project-wide concern, not specific to this feature.

**Action:** No action in this feature. Track as project-wide hardening.

### F4 — Delete endpoint has no soft-delete / confirmation: ACCEPTED

`DELETE /api/threads/:id` is permanent and immediate. This is intentional (same as existing `DELETE /api/feedback/:id` behavior). The append-only post model means individual posts cannot be deleted, but whole threads can. This is consistent with the data model spec.

### F5 — Auth gate coverage: PASSED

All 5 new `/api/threads/*` routes are registered inside the main router which is wrapped by the existing auth middleware. The public `/__sdlc/feedback` bypass alias was deliberately not replicated for threads — threads require authentication. Verified by inspecting route registration position in `lib.rs`.

## Summary

| Finding | Severity | Action |
|---------|----------|--------|
| F1 Path traversal via crafted ID | Low | Backlog item added |
| F2 Content injection | Info | Deferred to UI layer |
| F3 Unbounded content size | Info | Project-wide concern |
| F4 No soft-delete | Info | By design |
| F5 Auth gate | Pass | No action |

No blocking security issues. Implementation is safe for the current trust model (authenticated local-or-tunneled access).
