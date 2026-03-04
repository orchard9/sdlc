# Security Audit: Secrets — POST /api/secrets/envs create-only endpoint

## Scope

Focused audit of the `POST /api/secrets/envs` endpoint and the supporting `SecretEnvExists`
error variant. Examines authentication, input validation, information disclosure, filesystem
safety, and threat surface.

---

## A1 — Authentication bypass (FINDING: ACCEPTED — server-level control)

**Finding:** The endpoint is registered without any per-route authentication guard.

**Analysis:** The sdlc-server uses auth middleware at the application level
(`crates/sdlc-server/src/auth.rs`). The tunnel auth middleware gates all API routes globally —
clients must supply a valid token cookie or bearer token. The new route is registered in the same
router as all other `/api/secrets/*` routes and inherits the same middleware layer. No per-route
auth is needed.

**Verified:** `GET /api/secrets/envs` and `DELETE /api/secrets/envs/:name` use the same pattern
with no per-handler auth. Consistent.

**Action:** None.

---

## A2 — Path traversal in env name (FINDING: LOW RISK — acceptable)

**Finding:** The `env` field in the request body is not validated for path traversal characters
(`/`, `..`, null bytes).

**Analysis:** `sdlc_core::paths::secrets_env_path` produces:
```
<root>/.sdlc/secrets/envs/<env_name>.age
```
If `env_name` contains `/`, the resulting path would be `envs/sub/dir.age`, which could be
outside the expected directory. However:
1. The `age` encryption produces a file at the exact path — no shell expansion.
2. `sdlc_core::io::atomic_write` uses `std::fs::File::create`, which accepts the path as-is.
   A traversal like `../../evil` would write outside `.sdlc/secrets/envs/`, but only if the
   attacker controls the token (authenticated request).
3. The threat model is: authenticated user on their own developer machine. The server runs
   locally or through an authenticated tunnel. An attacker who has the tunnel token can already
   read all feature state.

**Risk level:** LOW. Authenticated attacker on a dev machine. Not a production multi-tenant service.

**Action:** Track as a future improvement. `sdlc task add secrets-create-env-endpoint "Validate env name: reject names containing / or .."` — but this is not a blocker.

Actually, this is recorded as F3 in the code review. No duplicate task needed.

---

## A3 — Secret values in server logs (FINDING: VERIFIED CLEAN)

**Finding:** The handler processes plaintext `KEY=VALUE` pairs. Could these be logged?

**Analysis:** The handler body is:
1. Deserialized from JSON by Axum — no logging.
2. The `body.pairs` are used to build the `KEY=VALUE` content string inside a
   `spawn_blocking` closure.
3. No `tracing::debug!` or `tracing::info!` calls reference `body.pairs` or `content`.
4. Axum's default request logging (if enabled) logs method/path/status — not request body.

The plaintext value exists only in memory within the blocking task and is passed directly to
`age` stdin for encryption. It is never written to disk in plaintext.

**Action:** None. Clean.

---

## A4 — Race condition in existence check (FINDING: ACCEPTED — same level as existing code)

**Finding:** The `.age` file existence check is performed before `write_env`, creating a TOCTOU
window.

**Analysis:** Covered in code review (F2). `write_env` calls `age` which creates the file. If two
concurrent requests create the same env, one succeeds and the other writes over the first (since
`atomic_write` uses a temp file + rename). The second request returns 201 even though a previous
version existed. This is the same TOCTOU behavior as `add_key` in the same module.

**Risk:** Cosmetic. Two users simultaneously creating the same env on the same developer machine is
not a realistic attack vector.

**Action:** None. Accepted.

---

## A5 — Missing input length/count limits (FINDING: LOW RISK)

**Finding:** No limits on the number of pairs or length of keys/values. A client could send
tens of thousands of pairs with multi-MB values, causing the server to allocate large amounts
of memory and spawn an expensive `age` invocation.

**Analysis:** The server runs locally for individual developers. There is no rate limiting on
any endpoint today (same for all existing routes). A motivated attacker with the tunnel token
could already exhaust resources through other routes. This is not a new vulnerability class.

**Action:** None at this time. Future hardening could add `max_pairs` and `max_pair_size`
validation, but it's out of scope for this feature.

---

## A6 — Encryption uses configured recipients (VERIFIED CORRECT)

**Finding:** Verify that encryption uses the configured recipient keys, not a hardcoded or
empty recipient list.

**Analysis:** The handler calls `sdlc_core::secrets::load_config(&root)` to obtain the current
keys, then passes `&config.keys` to `sdlc_core::secrets::write_env`. The no-keys check
(`if config.keys.is_empty()`) ensures encryption is never attempted with an empty recipient list.
`write_env` passes the keys as `-r` recipients to `age`. The encrypted output is unreadable
without the corresponding private key.

**Action:** None. Correct.

---

## A7 — 409 response does not leak env content (VERIFIED CLEAN)

**Finding:** When returning 409 Conflict, the error response could potentially disclose
information about the existing env (e.g., its key names).

**Analysis:** The 409 response is produced by `AppError`'s `IntoResponse` impl for
`SecretEnvExists`:
```json
{ "error": "env already exists: <env_name>" }
```
Only the env name (which the caller already provided) is disclosed. No key names, no content,
no metadata. Clean.

**Action:** None.

---

## Summary

| Finding | Severity | Action |
|---|---|---|
| A1: No per-route auth | N/A | Accepted — server-level control |
| A2: Path traversal in env name | LOW | Out of scope; tracked in F3 of code review |
| A3: Secret values in logs | NONE | Verified clean |
| A4: TOCTOU existence check | LOW | Accepted — same as existing code |
| A5: No input size limits | LOW | Out of scope; consistent with current server |
| A6: Encryption uses correct recipients | NONE | Verified correct |
| A7: 409 does not disclose content | NONE | Verified clean |

No blockers. The implementation is appropriate for the threat model (local dev server with
authenticated tunnel access).

## Verdict

APPROVED.
