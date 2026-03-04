# Security Audit: Secrets — Add Environment Modal and CLI Hint Affordances

## Scope

Changes in `SecretsPage.tsx`, `api/client.ts`, and a backend bug fix in `crates/sdlc-server/src/routes/secrets.rs`. The audit covers: client-side security surface of the new `AddEnvModal`, the `createSecretsEnv` API call, and the server-side `CreateEnvBody` deserialization fix (changed `pairs` from `HashMap<String, String>` to `Vec<EnvPair>` to match the frontend's array format).

## Findings

### F1: Secret values transmitted in plaintext over HTTP (within the browser session) — ACCEPTED

**Description:** The `createSecretsEnv` call sends `{ env, pairs: [{ key, value }] }` as JSON in the request body. The values are plaintext before encryption happens on the server.

**Assessment:** This is the same threat model as every other secrets UI. The API endpoint (`POST /api/secrets/envs`, delivered by `secrets-create-env-endpoint`) handles AGE encryption server-side using configured public keys. The server never stores plaintext. The channel is the local loopback or tunnel (TLS via orch-tunnel). This is consistent with how the existing `addSecretsKey` endpoint works and is accepted by design.

**Action:** Accepted — no change. Architecture explicitly documents that decryption is CLI-only and the server handles encryption.

---

### F2: Secret values visible in browser memory during modal use — ACCEPTED

**Description:** Key-value pairs are held as React state (`pairs: Pair[]`) during the modal session. They are cleared when the modal closes (component unmounts, state is garbage collected).

**Assessment:** React state in the browser is no more sensitive than any web form collecting a password. The values are not written to localStorage, sessionStorage, or any persistence layer by the frontend. Component unmount discards the state. This is acceptable for an in-browser form workflow.

**Action:** Accepted — no change.

---

### F3: No input sanitization on env name field — ACCEPTED

**Description:** The env name input is validated for non-empty but not for character set (alphanumeric/hyphen/underscore/dot as specified in FR-2). Character-set validation is enforced server-side.

**Assessment:** Server-side validation is the correct enforcement point for API inputs. Client-side character-set validation would be UX sugar. The spec noted this requirement as "validated as non-empty, alphanumeric/hyphen/underscore/dot" — the client enforces the empty check; the server enforces the character set. This is a standard split. No injection risk on the client side since the value is sent as a JSON string field.

**Action:** Accepted — server enforces character set; client enforces required. This matches the existing pattern for the key name field in `AddKeyModal`.

---

### F4: Copy button places secret in clipboard — ACCEPTED

**Description:** The `CopyButton` on the CLI set-secret hint (`sdlc secrets env set <env> KEY=value`) copies the literal text "KEY=value" (placeholder, not a real secret value). This is a static template hint, not an actual secret.

**Assessment:** No secret value is ever copied. The copy button copies the command template with the env name and a `KEY=value` placeholder. The user must substitute their own key and value at the terminal. No actual secret data passes through the clipboard.

**Action:** Accepted — no concern.

---

### F5: No CSRF protection on the new endpoint — ACCEPTED (pre-existing)

**Description:** `POST /api/secrets/envs` relies on the same auth middleware as all other API routes (token/cookie gate with local bypass). No per-request CSRF token is used.

**Assessment:** This is a pre-existing architectural posture consistent across all API routes. The auth middleware (token/cookie) provides a meaningful barrier. This feature does not introduce new CSRF surface beyond what already exists.

**Action:** Accepted — pre-existing, tracked separately from this feature.

---

## Verdict

No findings require blocking action. All identified items are either accepted by design (server-side encryption, same threat model as existing features) or pre-existing concerns outside this feature's scope. The implementation is safe to ship.
