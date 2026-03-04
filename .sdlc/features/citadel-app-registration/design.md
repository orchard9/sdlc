# Design: Register Citadel as a Pantheon App

## TLDR

Three new database tables in Pantheon (`app_registrations`, `tool_definitions`, `tool_credentials`), six REST routes, and a credential injection layer in Pantheon's existing HTTP dispatch pipeline. No changes to Citadel. No frontend changes in this feature.

---

## Architecture Overview

```
┌────────────────── Pantheon ──────────────────────────────────────────┐
│                                                                       │
│  REST API Layer                                                       │
│  ─────────────                                                        │
│  POST /organizations/:org/apps/register     → AppRegistrationHandler │
│  POST /organizations/:org/apps/:id/tools    → ToolDefinitionHandler  │
│  POST /organizations/:org/apps/:id/credentials → CredentialHandler   │
│  GET  /organizations/:org/apps/             → ListAppsHandler        │
│  GET  /organizations/:org/apps/:id/tools    → ListToolsHandler       │
│  DELETE /organizations/:org/apps/:id/credentials/:cid → DeleteCred  │
│                                                                       │
│  Data Layer (Postgres)                                                │
│  ─────────────────────                                                │
│  app_registrations  ──┐                                               │
│  tool_definitions   ──┤ FK: app_id                                   │
│  tool_credentials   ──┘ FK: app_id + org_slug                        │
│                                                                       │
│  Credential Service                                                   │
│  ──────────────────                                                   │
│  CredentialStore.Encrypt(orgSlug, plaintextKey) → encryptedBytes     │
│  CredentialStore.Decrypt(orgSlug, encryptedBytes) → plaintextKey     │
│  CredentialStore.Validate(key) → error (Citadel key format check)    │
│                                                                       │
│  Tool Execution (existing pipeline, extended)                         │
│  ─────────────────────────────────────────────                        │
│  ToolExecutor.Execute(orgSlug, toolName, params)                      │
│    → Resolve ToolDefinition by name                                   │
│    → Resolve ToolCredential by (appID, orgSlug)                       │
│    → Decrypt credential                                               │
│    → Build HTTP request: base_url + path_template + params           │
│    → Inject auth header                                               │
│    → Inject author_type: "ai_agent" for POST /annotations            │
│    → Execute HTTP request to Citadel                                  │
│    → Return response                                                  │
└───────────────────────────────────────────────────────────────────────┘
               │
               │  HTTPS  (Bearer ck_prod_... or X-Api-Key)
               ▼
┌────────── Citadel ─────────────────┐
│  GET  /api/v1/query                │
│  POST /api/v1/annotations          │
└────────────────────────────────────┘
```

---

## Database Schema

### Migration: `20260303_app_platform_foundation`

```sql
CREATE TABLE app_registrations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_slug    TEXT NOT NULL,
    name        TEXT NOT NULL,
    base_url    TEXT NOT NULL,
    auth_scheme TEXT NOT NULL CHECK (auth_scheme IN ('api_key', 'bearer_jwt', 'hmac')),
    auth_header TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (org_slug, name)
);

CREATE TABLE tool_definitions (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id            UUID NOT NULL REFERENCES app_registrations(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    description       TEXT NOT NULL,
    method            TEXT NOT NULL CHECK (method IN ('GET', 'POST', 'PATCH', 'DELETE')),
    path_template     TEXT NOT NULL,
    input_schema      JSONB NOT NULL,
    output_schema     JSONB,
    requires_approval BOOLEAN NOT NULL DEFAULT FALSE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, name)
);

CREATE TABLE tool_credentials (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id         UUID NOT NULL REFERENCES app_registrations(id) ON DELETE CASCADE,
    org_slug       TEXT NOT NULL,
    encrypted_key  BYTEA NOT NULL,
    key_prefix     TEXT NOT NULL,   -- first 8 chars (e.g. "ck_prod_")
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at     TIMESTAMPTZ,
    UNIQUE (app_id, org_slug)       -- one active credential per org per app
);

CREATE INDEX idx_tool_definitions_app_id ON tool_definitions(app_id);
CREATE INDEX idx_tool_credentials_app_org ON tool_credentials(app_id, org_slug);
```

---

## Go Package Structure

```
pantheon/
  internal/
    apps/
      handler.go         — HTTP handlers for all 6 routes
      model.go           — AppRegistration, ToolDefinition, ToolCredential structs
      store.go           — DB queries: create, list, get, delete
      credentials.go     — CredentialStore: encrypt/decrypt/validate
      executor.go        — ToolExecutor: resolves + dispatches HTTP calls
      executor_test.go   — Unit tests with mock HTTP server
      credentials_test.go — Encrypt/decrypt round-trip + key validation tests
```

---

## Credential Encryption Design

### Key Derivation

```
master_secret = os.Getenv("PANTHEON_CREDENTIAL_KEY")  // 32 bytes, base64
                                                        // fatal if absent at startup
org_key = HKDF-SHA256(
    secret:  master_secret,
    salt:    sha256(orgSlug),   // deterministic, no storage needed
    info:    "pantheon-tool-credential-v1",
    length:  32
)
```

### Encryption

```
nonce = crypto/rand 12 bytes
ciphertext = AES-256-GCM(key=org_key, nonce=nonce, plaintext=apiKey)
stored = nonce || ciphertext     // prepend nonce for self-contained decryption
```

### Why per-org key derivation?

- Compromising one org's encrypted blob doesn't help with another org's
- No key-per-credential storage needed — deterministic from master + org
- Master key rotation = re-encrypt all credentials (a maintenance task, not an emergency)

### Citadel Key Validation

```go
var citadelKeyPattern = regexp.MustCompile(
    `^ck_(prod|staging|dev)_[a-z0-9]{1,16}_[0-9a-f]{32}$`,
)

func validateCitadelKey(key string) error {
    if !citadelKeyPattern.MatchString(key) {
        return ErrInvalidCitadelKeyFormat
    }
    return nil
}
```

---

## HTTP Execution Flow

```go
func (e *ToolExecutor) Execute(ctx context.Context, orgSlug, toolName string, params map[string]any) (json.RawMessage, error) {
    // 1. Resolve tool definition
    tool, err := e.store.GetToolByName(ctx, toolName)
    if err != nil { return nil, err }

    // 2. Resolve credential
    cred, err := e.store.GetCredential(ctx, tool.AppID, orgSlug)
    if errors.Is(err, ErrCredentialNotFound) {
        return nil, fmt.Errorf("no Citadel credential configured for org %q: %w", orgSlug, err)
    }

    // 3. Decrypt
    apiKey, err := e.creds.Decrypt(orgSlug, cred.EncryptedKey)
    if err != nil { return nil, fmt.Errorf("credential decrypt failed: %w", err) }

    // 4. Build URL
    app, err := e.store.GetApp(ctx, tool.AppID)
    url := app.BaseURL + tool.PathTemplate

    // 5. Build request body / query params from input schema
    req, _ := e.buildRequest(tool.Method, url, tool.InputSchema, params)

    // 6. Inject auth header
    req.Header.Set(app.AuthHeader, "Bearer "+apiKey)

    // 7. Inject author_type for annotation calls (Citadel-specific)
    if tool.Name == "citadel_annotate_log" {
        // Merge into request body
        injectField(req, "author_type", "ai_agent")
    }

    // 8. Execute
    resp, err := e.http.Do(req)
    ...
    return body, nil
}
```

Note: The `author_type: "ai_agent"` injection in step 7 is intentionally Citadel-specific for now. When a second annotating app is registered, this should be generalized to an `inject_fields` map in `AppRegistration`.

---

## API Request/Response Shapes

### POST /api/v1/organizations/:org/apps/register

Request:
```json
{
  "name": "Citadel",
  "base_url": "https://citadel-staging.orchard9.ai",
  "auth_scheme": "bearer_jwt",
  "auth_header": "Authorization"
}
```
Response `201`:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "org_slug": "orchard9",
  "name": "Citadel",
  "base_url": "https://citadel-staging.orchard9.ai",
  "auth_scheme": "bearer_jwt",
  "auth_header": "Authorization",
  "created_at": "2026-03-03T09:00:00Z"
}
```

### POST /api/v1/organizations/:org/apps/:id/credentials

Request:
```json
{ "api_key": "ck_prod_orchard9_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6" }
```
Response `201` (plaintext key is NOT returned):
```json
{
  "id": "7f3a9c2e-...",
  "app_id": "550e8400-...",
  "org_slug": "orchard9",
  "key_prefix": "ck_prod_",
  "created_at": "2026-03-03T09:01:00Z"
}
```
Error `422` for invalid key format:
```json
{ "error": "invalid_citadel_key_format", "message": "Key must match ck_(prod|staging|dev)_<org>_<32hex>" }
```

---

## Sequence Diagram: Agent Queries Citadel Logs

```
Agent       Pantheon API    ToolExecutor    CredentialStore    Citadel
  │               │               │               │               │
  │ POST /execute │               │               │               │
  │ citadel_query │               │               │               │
  │──────────────►│               │               │               │
  │               │ Execute(org,  │               │               │
  │               │  tool, params)│               │               │
  │               │──────────────►│               │               │
  │               │               │ Decrypt(org,  │               │
  │               │               │  encKey)      │               │
  │               │               │──────────────►│               │
  │               │               │◄─────── apiKey│               │
  │               │               │               │               │
  │               │               │ GET /api/v1/query             │
  │               │               │ Authorization: Bearer ck_...  │
  │               │               │──────────────────────────────►│
  │               │               │◄────────────── log entries ───│
  │               │◄─────────────── log entries  │               │
  │◄──────────────│               │               │               │
```

---

## Error Handling

| Condition | HTTP Status | Error Code |
|---|---|---|
| Invalid key format | 422 | `invalid_citadel_key_format` |
| Credential not found during execution | 401 | `credential_not_configured` |
| Citadel API timeout | 504 | `upstream_timeout` |
| Citadel returns 401 | 502 | `upstream_auth_failed` |
| Decryption failure (key rotation?) | 500 | `credential_decrypt_failed` |
| Duplicate app name for org | 409 | `app_name_conflict` |

---

## Testing Strategy

1. **Unit tests** (`credentials_test.go`):
   - Encrypt → decrypt round-trip produces original key
   - Different orgs produce different ciphertext for same key
   - HKDF derivation is deterministic
   - `validateCitadelKey` accepts valid formats, rejects invalid

2. **Unit tests** (`executor_test.go`):
   - Mock HTTP server verifies auth header injection
   - `author_type: "ai_agent"` injected for annotate calls
   - Credential-not-found returns 401 to caller
   - HTTP 504 on upstream timeout

3. **Integration tests** (against Postgres test DB):
   - Full CRUD cycle: register → add tools → add credential → list → delete
   - UNIQUE constraint: duplicate app name per org fails with 409
   - Cascade delete: deleting app removes tools and credentials

---

## Migration and Rollout

- Migration runs automatically on Pantheon startup (Go-migrate or similar)
- `PANTHEON_CREDENTIAL_KEY` must be set in environment before first credential is stored
- Startup check: if credentials exist and key is absent, fail with explicit message
- No data seeding at deploy time — Citadel app registration is done via API after deploy

---

## Future Extensions (Not In This Feature)

- `inject_fields` map in `AppRegistration` to generalize `author_type` injection
- Frontend UI for managing app registrations
- Credential rotation endpoint (`PATCH .../credentials/:id/rotate`)
- Approval gate configuration per tool
- Multi-credential support per org (staging vs prod Citadel instances)
