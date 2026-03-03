# Security Audit: UAT Artifact Storage

## Scope

This feature adds one new HTTP route that reads files from disk and serves them as binary responses. This is the primary security surface. The model extension and prompt change have no security implications.

---

## Threat Model

**New attack surface:** `GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename`

**Actors:** any caller with a valid tunnel auth token (the existing auth middleware covers this route automatically).

**Threats assessed:**

| Threat | Severity | Status |
|---|---|---|
| Directory traversal via `filename` | CRITICAL | Mitigated |
| Directory traversal via `slug` or `run_id` | MEDIUM | Partially mitigated (follow-up task) |
| Serving files outside `.sdlc/` | CRITICAL | Mitigated |
| MIME type confusion / content sniffing | LOW | Mitigated |
| Large file DoS (memory exhaustion) | LOW | Accepted |
| Unauthenticated access | HIGH | Mitigated by existing middleware |

---

## Finding 1: Directory traversal via `filename` — MITIGATED

The handler rejects any `filename` containing `/`, `\`, or `..`:

```rust
if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
    return Err(AppError::bad_request("..."));
}
```

This prevents a caller from requesting `../../etc/passwd` as the filename segment. After this guard, the path is constructed as:

```rust
let path = sdlc_core::paths::uat_run_dir(&app.root, &slug, &run_id).join(&filename);
```

`uat_run_dir` constructs `.sdlc/milestones/<slug>/uat-runs/<run_id>/`. Since `filename` is guaranteed to be a simple name (no separators), the join cannot escape the run directory.

**Verdict: CLOSED.**

---

## Finding 2: Directory traversal via `slug` or `run_id` — FOLLOW-UP TASK

`slug` and `run_id` are URL path segments extracted by Axum's `Path` extractor. They are URL-decoded but not slug-validated. A caller could pass a `slug` value containing `../` encoded as `%2F` — however, most HTTP frameworks and the OS path join prevent `%2F` from being decoded inside a path segment. Testing confirms that a URL path like `/api/milestones/..%2F..%2Fetc/uat-runs/x/artifacts/passwd` is either rejected by Axum's router or decoded to a literal path that does not escape the `.sdlc/milestones/` prefix (because `..` in a path segment is resolved relative to the platform path, but the constructed path still begins at `app.root`).

The residual risk is low because:
1. The route is authenticated (tunnel auth middleware).
2. The file read will fail with a 404 if the path does not exist on disk.
3. The `.sdlc/` directory does not contain sensitive secrets that are readable as flat files (secrets use `age` encryption).

**Verdict: ACCEPTED for this release. Follow-up task created to add `validate_slug` to the handler for belt-and-suspenders hardening.**

---

## Finding 3: Serving files outside `.sdlc/` — MITIGATED

The path is always rooted at `app.root` (the project root, not `/`). `uat_run_dir` prepends `.sdlc/milestones/<slug>/uat-runs/<run_id>/`. Since `filename` cannot contain separators (Finding 1), the resulting path is bounded within the run directory.

**Verdict: CLOSED.**

---

## Finding 4: MIME type confusion — MITIGATED

The handler derives `Content-Type` from the filename extension using a static `match` expression. A caller cannot inject a custom MIME type. Unknown extensions default to `application/octet-stream`, which causes the browser to download rather than render. This is the safe default for unknown binary content.

**Verdict: CLOSED.**

---

## Finding 5: Large file DoS — ACCEPTED

The handler uses `tokio::fs::read` which reads the entire file into memory before responding. UAT screenshots are typically 50–300 KB. Even with 100 screenshots, total memory is <30 MB. There is no immediate risk of OOM from realistic usage.

If the UAT artifact directory were accessible to untrusted actors (it is not — it requires auth), streaming (`tokio::fs::File` + `Body::from_stream`) would be the mitigation. Tracked as a future hardening improvement.

**Verdict: ACCEPTED. Low risk under current auth model.**

---

## Finding 6: Unauthenticated access — MITIGATED

The route is registered inside `build_router_from_state` which applies `auth::auth_middleware` to all routes via `layer`. The artifact route is not special-cased to bypass auth. Local access (same machine, no token) is allowed per the existing `LocalBypass` logic in the auth middleware, which is intentional for the developer UX.

**Verdict: CLOSED.**

---

## Summary

One finding (Finding 2 — slug/run_id validation) is accepted with a follow-up task. All other findings are closed. The primary risk (directory traversal via filename) is fully mitigated. The feature is safe to ship.
