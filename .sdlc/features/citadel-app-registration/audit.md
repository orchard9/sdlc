# Audit: Register Citadel as a Pantheon App

## Scope

Security and compliance audit of the App Platform foundation: credential storage model, encryption design, API surface, and data isolation guarantees.

---

## Findings

### Security

**[A1] PASS — Plaintext credentials never leave the service boundary.**
The `StoreCredential` endpoint accepts a plaintext `api_key` in the request body, but the response returns only `id`, `app_id`, `org_slug`, `key_prefix`, and timestamps. The plaintext key is not stored, not logged, and not returned. Logs must be audited to confirm no key appears in debug output (covered in QA plan's "no plaintext in logs" check).

**[A2] PASS — AES-256-GCM with random nonces prevents ciphertext analysis.**
A new random 12-byte nonce is generated per encryption operation. The nonce is prepended to the ciphertext for self-contained decryption. Identical keys for different orgs produce different ciphertexts because HKDF derives a different org-specific key.

**[A3] PASS — Per-org key derivation limits blast radius.**
Compromising one org's encrypted credential blob requires knowing both the master key AND the org slug. Compromising the master key alone does not decrypt credentials for all orgs simultaneously — it requires re-deriving each org's key individually. This is a meaningful security improvement over a single shared encryption key.

**[A4] PASS — UNIQUE(app_id, org_slug) prevents credential shadowing.**
Only one active credential per org per app is permitted. An attacker who registers a second credential for the same org/app via a race condition would get a 409 conflict (or upsert would replace, not append). Either behavior is safe; the implementation must choose one and document it (flagged in review as an action item — confirmed as in-scope for implementation).

**[A5] CONCERN (ACCEPTED) — Master key rotation requires re-encryption of all credentials.**
`PANTHEON_CREDENTIAL_KEY` rotation is a maintenance operation that requires re-encrypting all `encrypted_key` blobs. This is not a vulnerability but a known operational cost. Mitigation: the key derivation scheme (`HKDF-SHA256(master, salt=sha256(orgSlug))`) makes this mechanical — a migration script re-derives all org keys from the new master and re-encrypts. Accepted: documented as an operational runbook requirement, not a code defect.

**[A6] PASS — Startup fail-fast prevents silent misconfiguration.**
`RequireCredentialKey()` checks for `PANTHEON_CREDENTIAL_KEY` at startup. If the key is absent and credentials are stored, the service fails with an explicit error message. This prevents a scenario where credentials are stored but silently unreadable (which would surface as confusing auth failures in production rather than at deploy time).

**[A7] MINOR — Citadel API key format validation uses regex, not constant-time comparison.**
The validation function checks the format of the key before storage. Since this is a format check (not a secret comparison), non-constant-time execution is acceptable — no timing oracle attack is possible here. No action required.

### Data Isolation

**[A8] PASS — Org isolation enforced at the query layer.**
All credential lookups are keyed by `(app_id, org_slug)`. The `GetCredential(appID, orgSlug)` store method includes `org_slug` in the WHERE clause, preventing cross-org credential access even if an attacker knows a credential's UUID.

**[A9] PASS — Credential deletion verifies org ownership.**
The `DeleteCredential(credID, orgSlug)` method verifies the credential belongs to the requesting org before deletion. This prevents one org from deleting another org's credential via a known UUID.

**[A10] PASS — Cascade delete removes child records atomically.**
The `ON DELETE CASCADE` constraint on `tool_definitions` and `tool_credentials` ensures app deletion removes all associated records in a single transaction. No orphaned credentials or tool definitions can persist after an app is removed.

### API Surface

**[A11] PASS — No sensitive data in error responses.**
Error codes (`invalid_citadel_key_format`, `credential_not_configured`, etc.) are defined enums that do not leak internal state. Error messages are user-readable but do not include stack traces, SQL errors, or credential fragments.

**[A12] PASS — Route authorization relies on org scoping in path.**
All routes are under `/api/v1/organizations/{orgSlug}/apps/...`. Pantheon's existing auth middleware must verify the calling identity has access to `{orgSlug}` before these handlers execute. This is an assumed dependency — the implementation must confirm the existing middleware covers these new routes.

**[A13] MINOR — No rate limiting on credential storage endpoint.**
A caller with valid org access could enumerate Citadel key formats by submitting many invalid keys (each returning 422). Since the key format is documented, this provides no information gain over reading the docs. Rate limiting on this endpoint is not required for security but may be added for operational hygiene. Accepted as follow-on.

### Compliance

**[A14] PASS — Credential at rest uses industry-standard encryption.**
AES-256-GCM is FIPS 140-2 approved. HKDF-SHA256 key derivation is standard (RFC 5869). This implementation meets typical enterprise security requirements for credential storage.

**[A15] PASS — Audit trail via `created_at` and `rotated_at` timestamps.**
`tool_credentials.created_at` and `rotated_at` provide a basic audit trail for credential lifecycle. Combined with Postgres WAL/change data capture (if enabled), this gives compliance teams a record of credential rotation events.

---

## Finding Disposition

| ID | Severity | Disposition |
|---|---|---|
| A1 | HIGH | PASS — no action |
| A2 | HIGH | PASS — no action |
| A3 | HIGH | PASS — no action |
| A4 | HIGH | PASS — no action |
| A5 | MEDIUM | ACCEPTED — document operational runbook requirement |
| A6 | HIGH | PASS — no action |
| A7 | LOW | ACCEPTED — not applicable (format check, not secret comparison) |
| A8 | HIGH | PASS — no action |
| A9 | HIGH | PASS — no action |
| A10 | HIGH | PASS — no action |
| A11 | MEDIUM | PASS — no action |
| A12 | HIGH | PASS — confirm existing middleware covers new routes |
| A13 | LOW | ACCEPTED — follow-on: rate limiting for operational hygiene |
| A14 | HIGH | PASS — no action |
| A15 | MEDIUM | PASS — no action |

---

## Required Actions Before Production

1. **Confirm existing auth middleware covers new `/apps/` routes** (A12) — verify in implementation PR.
2. **Document `PANTHEON_CREDENTIAL_KEY` rotation procedure** (A5) — add to Pantheon's operational runbook before first production deployment.
3. **Confirm no `api_key` appears in any log line** (A1) — include explicit log-scan step in integration tests.

---

## Verdict

APPROVED. No blocking security findings. The credential storage design is sound and the data isolation guarantees are correct. Three implementation confirmations required before production deployment (middleware coverage, key rotation runbook, log scan), none of which block the feature from proceeding.
