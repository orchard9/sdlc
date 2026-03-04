# Tasks: Register Citadel as a Pantheon App

## T1 — DB migration: `app_registrations`, `tool_definitions`, `tool_credentials` tables

Write and apply the migration `20260303_app_platform_foundation.sql`:
- `app_registrations` table with `(id, org_slug, name, base_url, auth_scheme, auth_header, created_at)` and `UNIQUE(org_slug, name)` constraint
- `tool_definitions` table with `(id, app_id FK, name, description, method, path_template, input_schema JSONB, output_schema JSONB, requires_approval, created_at)` and `UNIQUE(app_id, name)` constraint
- `tool_credentials` table with `(id, app_id FK, org_slug, encrypted_key BYTEA, key_prefix, created_at, rotated_at)` and `UNIQUE(app_id, org_slug)` constraint
- Indexes: `idx_tool_definitions_app_id`, `idx_tool_credentials_app_org`
- CASCADE DELETE from `app_registrations` to both child tables

## T2 — Go data model and store (`internal/apps/model.go`, `store.go`)

Define structs: `AppRegistration`, `ToolDefinition`, `ToolCredential`.
Implement `Store` interface with methods:
- `CreateApp(ctx, AppRegistration) → AppRegistration, error`
- `GetApp(ctx, appID) → AppRegistration, error`
- `ListApps(ctx, orgSlug) → []AppRegistration, error`
- `CreateToolDefinition(ctx, ToolDefinition) → ToolDefinition, error`
- `ListToolDefinitions(ctx, appID) → []ToolDefinition, error`
- `GetToolByName(ctx, name) → ToolDefinition, error`
- `CreateCredential(ctx, ToolCredential) → ToolCredential, error`
- `GetCredential(ctx, appID, orgSlug) → ToolCredential, error`
- `DeleteCredential(ctx, credID, orgSlug) → error`

## T3 — Credential service (`internal/apps/credentials.go`)

Implement `CredentialStore` with:
- `Encrypt(orgSlug, plaintextKey string) → ([]byte, error)` — HKDF-SHA256 key derivation from `PANTHEON_CREDENTIAL_KEY` env var, then AES-256-GCM with random nonce (nonce prepended to ciphertext)
- `Decrypt(orgSlug string, ciphertext []byte) → (string, error)` — reverse of Encrypt
- `Validate(key string) → error` — regex match `^ck_(prod|staging|dev)_[a-z0-9]{1,16}_[0-9a-f]{32}$`
- Startup check function `RequireCredentialKey() error` — fatal if `PANTHEON_CREDENTIAL_KEY` is not set
- Write `credentials_test.go`: round-trip, different orgs produce different ciphertext, deterministic derivation, key validation

## T4 — HTTP handlers (`internal/apps/handler.go`)

Implement the 6 REST handlers wired to `Store` and `CredentialStore`:
- `RegisterApp` — POST, creates AppRegistration
- `CreateTool` — POST, creates ToolDefinition
- `StoreCredential` — POST, validates key format, encrypts, stores (returns id + key_prefix only)
- `ListApps` — GET, lists apps for org
- `ListTools` — GET, lists tools for app
- `DeleteCredential` — DELETE, removes credential (verifies org ownership)
Register routes in Pantheon's router under `/api/v1/organizations/:orgSlug/apps/...`

## T5 — Tool executor extension (`internal/apps/executor.go`)

Extend or create `ToolExecutor`:
- `Execute(ctx, orgSlug, toolName string, params map[string]any) → json.RawMessage, error`
- Resolves tool definition, resolves credential, decrypts, builds and fires HTTP request
- Injects `Authorization: Bearer <key>` (or custom auth_header)
- Injects `author_type: "ai_agent"` into request body for `citadel_annotate_log` calls
- Handles upstream 401 → returns `ErrUpstreamAuthFailed`; upstream timeout → returns `ErrUpstreamTimeout`
- Write `executor_test.go` with mock HTTP server covering: successful round-trip, credential-not-found, auth header injection, author_type injection, upstream error mapping

## T6 — Seed Citadel app and tool definitions via integration test/fixture

Write a test or seed function that calls the API to:
1. Register Citadel as an app (`name: "Citadel"`, `base_url: "https://citadel-staging.orchard9.ai"`, `auth_scheme: "bearer_jwt"`, `auth_header: "Authorization"`)
2. POST `citadel_query_logs` tool definition (per spec JSON schema)
3. POST `citadel_annotate_log` tool definition (per spec JSON schema)
Verify both tool definitions are returned via GET and the JSON schemas are stored correctly.

## T7 — Integration tests (Postgres test DB)

Write integration tests covering:
- Full CRUD: register app → add 2 tools → add credential → list apps/tools → delete credential
- UNIQUE constraint violation: duplicate `(org_slug, name)` returns HTTP 409
- CASCADE: deleting app removes its tools and credentials
- Credential isolation: org A cannot retrieve org B's credential
- `StoreCredential` with invalid key format returns HTTP 422 with correct error code
