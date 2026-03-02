# Security Audit: Edit Feedback Notes Inline

## Threat Surface

This feature adds a `PATCH /api/feedback/:id` endpoint and an inline edit UI. The security surface is narrow — it extends an existing authenticated internal API with one new mutation endpoint.

## Authentication and Authorization

**Finding A1 — Route is behind existing auth middleware: PASS**

The `PATCH /api/feedback/{id}` route is registered in the same `Router` that applies the auth middleware defined in `crates/sdlc-server/src/auth.rs`. There is no bypass. The existing `DELETE` route on the same path pattern is identically protected — this new route inherits the same gate.

**Finding A2 — No tenant separation needed: PASS**

`sdlc` runs as a single-user local tool. All feedback notes belong to the same user/project. No user-scoping of note IDs is required.

## Input Validation

**Finding A3 — Empty content rejected at handler boundary: PASS**

`update_note` checks `body.content.trim().is_empty()` and returns 400 before calling the core layer. This prevents empty-string writes to disk.

**Finding A4 — No length cap on content: ACCEPTED**

The `add_note` handler similarly has no length cap on note content. Consistency with the existing `add_note` behaviour is preferred over introducing an asymmetry. The risk is bounded: notes live in `.sdlc/feedback.yaml` on the local filesystem; an adversarial oversize write would require authenticated access to the running server. Tracking in a future hardening pass.

**Finding A5 — ID path parameter is unvalidated beyond string type: PASS**

The `id` is passed to `sdlc_core::feedback::update()`, which does a linear scan by string equality (`n.id == id`). No filesystem path construction uses the ID directly — the feedback store is a single flat YAML file. No path traversal risk.

## Data Integrity

**Finding A6 — Atomic write used for persistence: PASS**

`save_all` calls `io::atomic_write` which writes to a temp file and renames atomically. No partial writes possible.

**Finding A7 — Optimistic update on frontend, server authoritative on failure: PASS**

The frontend applies an optimistic update and calls `load()` on network error to restore server state. No stale client data persists on failure.

**Finding A8 — YAML injection via note content: PASS**

`serde_yaml::to_string` serialises the content field with proper quoting/escaping. Adversarial content in the `content` string cannot break YAML structure.

## Frontend

**Finding A9 — No XSS risk in edit UI: PASS**

The `<textarea>` element handles the edit draft as a controlled React value. Content is displayed via `<pre>{note.content}</pre>` (text node, not `dangerouslySetInnerHTML`). No user-controlled HTML is injected into the DOM.

**Finding A10 — Save button disabled on empty draft: PASS**

`disabled={editSaving || !editDraft.trim()}` prevents zero-length PATCH requests from the UI.

## Summary

| Finding | Severity | Status |
|---|---|---|
| A1 — Auth middleware covers new route | — | PASS |
| A2 — No tenant separation needed | — | PASS |
| A3 — Empty content rejected 400 | — | PASS |
| A4 — No max-length cap on content | LOW | ACCEPTED (consistent with add_note, local tool) |
| A5 — ID path param, no traversal | — | PASS |
| A6 — Atomic write | — | PASS |
| A7 — Optimistic update with server restore | — | PASS |
| A8 — YAML injection via serde | — | PASS |
| A9 — No XSS in edit UI | — | PASS |
| A10 — Save disabled on empty draft | — | PASS |

No blocking security findings. One low-severity accepted finding (A4) noted for future hardening.
