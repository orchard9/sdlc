# Security Audit: Inline Secret Rotation UI

## Threat Model

This feature adds a `PATCH /api/secrets/envs/:name` endpoint that re-encrypts a secrets env file with caller-supplied plaintext key-value pairs. The security surface is the new HTTP endpoint and the plaintext in transit.

## Findings

### 1. Plaintext secrets in HTTP request body — ACCEPTED (by design, TLS required)

**Finding**: The PATCH request body contains plaintext `key=value` pairs over HTTP. Anyone with access to the request can read the secret values.

**Mitigation**: The server is accessed either (a) locally (no network exposure) or (b) via the tunnel (`orch-tunnel`) which enforces TLS. Auth tokens gate all tunnel access. This is the same risk surface as the existing `POST /api/secrets/envs` endpoint, which already accepts plaintext. No new exposure introduced.

**Action**: ACCEPTED. Document in deployment guidance that tunnel must use TLS (already required).

### 2. Authorization: no per-env ACL — ACCEPTED (same as existing endpoints)

**Finding**: Any authenticated user (valid tunnel token) can PATCH any env by name. There is no per-environment access control.

**Mitigation**: The auth token system is binary (authenticated = full access). All existing secrets endpoints have the same property. Per-env ACL is out of scope for this feature.

**Action**: ACCEPTED. Track as future hardening if multi-user access is added.

### 3. PATCH is unauthenticated on local access — ACCEPTED (same as all endpoints)

**Finding**: When running locally (no tunnel), all endpoints including the new PATCH are unauthenticated. An attacker with local network access could update secrets.

**Mitigation**: Local-only access is a conscious security boundary for the dev use case. This is identical to `DELETE /api/secrets/envs/:name` which can destroy an entire env.

**Action**: ACCEPTED. No change.

### 4. Secret values appear in server logs — NO ISSUE

**Finding**: Do server logs capture request bodies?

**Verification**: The `log_request` middleware in `lib.rs` logs `method`, `path`, and `status` only — not request body. No plaintext exposure via logs.

**Action**: NO ISSUE.

### 5. Input validation: key name injection — ACCEPTED (contained by age encryption)

**Finding**: Key names and values are passed to `write_env()` which formats `KEY=VALUE` and feeds it to the `age` binary via stdin. Could a crafted key name inject shell metacharacters?

**Verification**: The content is passed via stdin (not command-line args). The `age` binary reads stdin as raw bytes. No shell interpolation occurs. The formatted string `format!("{}={}", key, value)` is safe for env file format.

**Action**: ACCEPTED. No shell injection vector.

### 6. Atomicity: partial write on failure — NO ISSUE

**Finding**: If `write_env()` fails mid-write, could the env file be left in a corrupted state?

**Verification**: `write_env()` uses `io::atomic_write()` which writes to a temp file and renames — standard atomic write pattern. On failure, the original `.age` file is unchanged.

**Action**: NO ISSUE.

## Summary

| Finding | Severity | Action |
|---|---|---|
| Plaintext in request body | INFO | Accepted — same as POST, TLS-gated |
| No per-env ACL | INFO | Accepted — same as all endpoints |
| Unauthenticated on local | INFO | Accepted — same as DELETE |
| Logs capture body | NONE | Not applicable |
| Key name injection | INFO | Accepted — no shell vector |
| Partial write atomicity | NONE | Protected by atomic_write |

**Audit verdict**: APPROVED. No new security vulnerabilities. All findings match the existing security posture of the secrets subsystem.
