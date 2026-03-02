# Security Audit: Blocked Feature UX — BlockedPanel

## Scope

New surface: `DELETE /api/features/:slug/blockers/:idx` endpoint and the frontend
`BlockedPanel` component that calls it.

---

## Threat model

### 1. Index manipulation — BLOCKER: None / LOW risk

**Concern:** A caller sends `idx=0` for a feature with many blockers, removing the wrong one.

**Analysis:** The `remove_blocker` method removes the element at the given index. Since
blockers are a plain `Vec<String>`, index semantics are stable within a single request.
The only risk is a concurrent race where two simultaneous DELETE requests target different
intended blockers but the same index after one is already removed. For a single-user SDLC
tool (no multi-tenant sharing model), this is not a meaningful attack surface.

**Decision:** Accept. Blockers are non-sensitive — removing the wrong one is a UX
inconvenience, not a security issue. The blocker list is fully visible in the UI.

### 2. Unauthenticated feature mutation — LOW risk (consistent with existing surface)

**Concern:** The DELETE endpoint can mutate feature state without authentication.

**Analysis:** The existing `/api/features/:slug` endpoints — `POST /transition`,
`POST /merge`, `POST /comments`, `POST /tasks` — all lack authentication in the same
way. This feature adds no new authentication requirement beyond what already exists.
`sdlc-server` uses the tunnel auth layer (`auth.rs`) for remote access; local access is
trusted by design (same machine). The DELETE endpoint is consistent with the existing
trust model.

**Decision:** Accept. No regression; consistent with the existing surface. Tracked
as a systemic concern for the auth layer separately.

### 3. Reason body injection — LOW risk

**Concern:** The `reason` field is stored as-is in a `decision` comment on the feature.
Could it be used to inject malicious content?

**Analysis:** The comment body is a plain `String` stored in YAML. It is displayed in
the frontend inside a `<p>` tag with no `dangerouslySetInnerHTML`. React's default
text escaping prevents XSS. The `reason` field is size-unconstrained, but:
- The server-side handler trims and rejects empty reasons.
- Comments are internal project state, not user-facing in a public context.
- Maximum practical size is bounded by the fetch body size limits.

**Decision:** Accept. No injection risk in the current rendering path.

### 4. Out-of-bounds index via large integers — LOW risk

**Concern:** Sending `idx=99999999999` could cause issues.

**Analysis:** Axum's `Path<(String, usize)>` extractor will reject values that don't
parse as `usize` (returns 422). Values that parse but are out of range are caught by
`remove_blocker`'s bounds check, which returns `SdlcError::InvalidPhase` → 400. No
panic path exists.

**Decision:** Accept. Handled correctly.

### 5. Frontend fetch without CSRF protection — INFORMATIONAL

**Concern:** The `fetch` call in `BlockedPanel` does not include a CSRF token.

**Analysis:** Same-origin `fetch` with no credentials is not CSRF-exploitable in the
relevant threat model (localhost dev tool + cloudflared tunnel with token). All existing
frontend fetch calls follow the same pattern. No regression.

**Decision:** Accept. Informational only; consistent with existing surface.

---

## Findings summary

| Finding | Severity | Action |
|---|---|---|
| Index manipulation race | Informational | Accept |
| No endpoint auth | Informational | Accept (systemic, not new) |
| Reason body injection | Informational | Accept |
| Large index out of bounds | Informational | Accept (handled) |
| No CSRF on fetch | Informational | Accept (consistent) |

No blocking or high-severity findings. All findings are informational and consistent
with the existing security posture of the `sdlc-server` tool.

---

## Verdict: APPROVED — no action required
