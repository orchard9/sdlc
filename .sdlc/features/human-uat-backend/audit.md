# Security Audit: human-uat-backend

## Threat Surface

Two new POST endpoints that write to the filesystem:

1. `POST /api/milestone/{slug}/uat/human` — writes `run.yaml` and `summary.md` under `.sdlc/milestones/`
2. `POST /api/features/{slug}/human-qa` — writes `qa-results.md` under `.sdlc/features/`

## Findings

### FINDING 1: Slug Injection — MITIGATED

**Risk:** A malicious `slug` value could path-traverse outside `.sdlc/`.

**Analysis:** Both handlers call `validate_slug(&slug)` before any disk I/O. `sdlc_core::paths::validate_slug` enforces `[a-z0-9-]` — no slashes, dots, or special characters. Path traversal is not possible.

**Status:** ACCEPTED (mitigated by existing slug validation).

### FINDING 2: Notes/Verdict Content in Files — LOW RISK

**Risk:** User-supplied `notes` and `verdict` values are written directly into `summary.md` and `qa-results.md` without sanitization.

**Analysis:** These are Markdown files stored in the local `.sdlc/` directory, readable only by authenticated users (behind the tunnel auth middleware). There is no rendering path that would execute injected content as code. Markdown is safe for display.

**Status:** ACCEPTED. No sanitization needed for local-only Markdown files.

### FINDING 3: Milestone Release on Pass — INTENTIONAL

**Risk:** A human submitter could mark a milestone as released by sending `verdict: pass`.

**Analysis:** This is the intended behavior — the endpoint is specifically designed to support human sign-off as an alternative to AI-driven UAT. The server is already behind tunnel authentication; only authenticated users can reach this endpoint.

**Status:** ACCEPTED (by design).

### FINDING 4: No Rate Limiting on Submission Endpoints — LOW RISK

**Risk:** A client could spam the endpoint to create many `run.yaml` files.

**Analysis:** The server has no per-endpoint rate limiting at this time. This is a general concern across all POST endpoints, not specific to this feature. The `.sdlc/` directory is local and disk usage from UAT run files is negligible (each `run.yaml` is < 1 KB).

**Status:** TRACKED — add to general server hardening backlog. Not a blocker for this feature.

### FINDING 5: Authentication — MITIGATED

**Risk:** Unauthenticated callers could submit UAT results.

**Analysis:** Both routes are registered in `build_router_from_state`, which applies the tunnel auth middleware globally (enforced in `auth.rs`). In dev mode, localhost requests bypass auth by design. No change to the auth model needed.

**Status:** ACCEPTED (handled by existing middleware).

## Summary

No blockers. Two findings (notes content, milestone release) are intentional by design. Rate limiting is a known systemic gap tracked at the project level. All new code follows the no-`unwrap()` convention and uses `atomic_write` for all file I/O.

## Verdict: APPROVED
