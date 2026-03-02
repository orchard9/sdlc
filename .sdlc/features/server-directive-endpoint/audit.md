# Security Audit: server-directive-endpoint

## Surface

`GET /api/features/{slug}/directive` — read-only endpoint, no writes, no side effects.

## Findings

**Input validation:** The slug path parameter is passed directly to `Feature::load`, which resolves it to a known directory under `.sdlc/features/`. No path traversal is possible since the load function sanitizes the slug to a flat directory name.

**Data exposure:** Returns the classification of a feature. This is the same data returned by `GET /api/features/{slug}/next`. No credentials, secrets, or sensitive user data are included.

**Authentication:** Subject to the same tunnel auth middleware as all other API routes. No special bypass.

**Injection:** No shell execution, no SQL, no template rendering. Pure in-memory classification read.

## Verdict: No Issues

This is a safe, additive read endpoint with the same risk profile as the existing `/next` route.
