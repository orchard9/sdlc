# QA Plan: Register Citadel as a Pantheon App

## Scope

This plan covers the App Platform foundation in Pantheon: three new database tables, six REST routes, the credential encryption/decryption service, and the tool executor HTTP dispatch layer.

The Citadel service itself is out of scope — tests use a mock HTTP server for Citadel calls.

---

## Test Layers

### Layer 1: Unit Tests (no external deps)

**Credential Service (`credentials_test.go`)**

| Test | Expected |
|---|---|
| `Encrypt` → `Decrypt` round-trip returns original key | Pass |
| Same key + different orgs produce different ciphertext | Pass |
| HKDF derivation is deterministic (same inputs = same org key) | Pass |
| `Validate("ck_prod_orchard9_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6")` | No error |
| `Validate("ck_staging_myorg_deadbeef..." )` (32 hex) | No error |
| `Validate("sk_prod_org_..." )` (wrong prefix) | ErrInvalidCitadelKeyFormat |
| `Validate("ck_prod_org")` (missing random segment) | ErrInvalidCitadelKeyFormat |
| `Validate("ck_prod_org_ABCDEF...")` (uppercase hex) | ErrInvalidCitadelKeyFormat |
| `Validate("ck_live_org_...")` (invalid env) | ErrInvalidCitadelKeyFormat |
| `Validate("ck_prod_toolong_org_slug_here_abc...")` (org > 16 chars) | ErrInvalidCitadelKeyFormat |
| `RequireCredentialKey()` with `PANTHEON_CREDENTIAL_KEY` unset | Returns error |
| `RequireCredentialKey()` with key set | Returns nil |

**Tool Executor (`executor_test.go`)**

| Test | Expected |
|---|---|
| Execute `citadel_query_logs` — mock returns 200 with log JSON | Returns log JSON |
| Auth header `Authorization: Bearer ck_...` injected on every call | Header present in mock |
| `author_type: "ai_agent"` injected into `citadel_annotate_log` POST body | Field present in captured request |
| `author_type` NOT injected into `citadel_query_logs` GET | Field absent |
| ToolDefinition not found → returns `ErrToolNotFound` | Error returned |
| Credential not found for org → returns `ErrCredentialNotFound` (HTTP 401 to caller) | Error mapped correctly |
| Mock Citadel returns 401 → `ErrUpstreamAuthFailed` | Error mapped correctly |
| Mock Citadel times out → `ErrUpstreamTimeout` | Error mapped correctly |

---

### Layer 2: Integration Tests (Postgres test DB)

**App Registration CRUD**

| Test | Expected |
|---|---|
| `POST /apps/register` with valid payload → 201, returns AppRegistration JSON | Pass |
| `POST /apps/register` duplicate `(org_slug, name)` → 409 `app_name_conflict` | Pass |
| `GET /apps/` returns all apps for org, zero for different org | Isolated correctly |

**Tool Definition CRUD**

| Test | Expected |
|---|---|
| `POST /apps/:id/tools` with valid JSON schema → 201 | Pass |
| `GET /apps/:id/tools` returns all tools for that app | Pass |
| `POST /apps/:id/tools` duplicate `(app_id, name)` → 409 | Pass |
| JSONB `input_schema` is stored and returned verbatim | Pass |

**Credential Storage**

| Test | Expected |
|---|---|
| `POST /apps/:id/credentials` with valid `ck_prod_...` key → 201, plaintext key NOT in response | Pass |
| `POST /apps/:id/credentials` with invalid key format → 422 `invalid_citadel_key_format` | Pass |
| `DELETE /apps/:id/credentials/:cid` → 204, subsequent exec returns 401 | Pass |
| Org A cannot read Org B's credential via list or direct query | Isolation confirmed |
| Duplicate credential for same `(app_id, org_slug)` → 409 or upsert depending on policy | Defined behavior |

**Cascade Deletion**

| Test | Expected |
|---|---|
| Delete `app_registrations` row → associated `tool_definitions` rows deleted | Pass |
| Delete `app_registrations` row → associated `tool_credentials` rows deleted | Pass |

**Startup Check**

| Test | Expected |
|---|---|
| Service starts without `PANTHEON_CREDENTIAL_KEY` when no credentials stored | Allowed (no stored creds) |
| Service fails startup if credentials exist and `PANTHEON_CREDENTIAL_KEY` is absent | Fatal error with clear message |

---

### Layer 3: Acceptance Criteria Verification (from spec)

| # | Criterion | How to verify |
|---|---|---|
| 1 | POST register returns AppRegistration | Integration test: inspect response JSON |
| 2 | POST tool persists ToolDefinition with correct JSON schema | Integration test: GET tools, compare schema |
| 3 | POST credential stores encrypted; plaintext never returned | Unit test: inspect response, confirm no `api_key` field |
| 4 | Encrypted credential is decryptable by execution layer | Executor unit test: full round-trip with mock server |
| 5 | Invalid key format → HTTP 422 | Integration test: send malformed key |
| 6 | Delete credential → subsequent exec returns 401 | Integration test: delete then execute |
| 7 | citadel_query_logs and citadel_annotate_log definitions can be seeded | Integration test: seed + GET + compare |

---

## Definition of Done

- All unit tests pass (no skips)
- All integration tests pass against Postgres test DB
- All 7 acceptance criteria verified
- `PANTHEON_CREDENTIAL_KEY` startup check documented in Pantheon's `.env.example`
- No plaintext API key appears in any log line (scan logs in integration test runs)
