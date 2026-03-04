# Spec: Register Citadel as a Pantheon App

## TLDR

Add Citadel as a registered application in Pantheon's App Platform. This means persisting a `ToolDefinition` record (JSON schema per tool endpoint) and a `ToolCredential` record (encrypted Citadel API key, org-scoped) in Pantheon's data layer, and wiring those into Pantheon's existing auth + HTTP execution pipeline so agents can call Citadel tools as first-class Pantheon capabilities.

---

## Background

Pantheon (Go service, Discord-first developer hub) has an existing App Platform with:
- `ToolDefinition` — a JSON schema description of an external API call a tool can make
- `ToolCredential` — encrypted API credentials scoped to a tool and org
- Tool execution — Pantheon handles auth injection and HTTP dispatch on behalf of agents
- Approval gates — human approval for high-risk calls

Citadel (Rust, enterprise observability) uses API key format `ck_<env>_<org>_<random>`, hashed with Argon2id, and carries them in either a `Bearer` JWT or the `X-Tenant-ID` header depending on trust level.

This feature registers Citadel as an App in Pantheon's platform so that Tier 2 agent tools (`citadel_query_logs`, `citadel_annotate_log`) can be executed without each caller handling credentials.

---

## Problem

Citadel tools cannot be registered or executed via Pantheon's App Platform today because:

1. No `ToolDefinition` records exist for Citadel endpoints.
2. No `ToolCredential` model exists with the Citadel key format (`ck_<env>_<org>_<random>`).
3. No org-scoped encrypted storage pattern exists for external service API keys.
4. Pantheon's HTTP execution pipeline has no mechanism to inject a Citadel API key into outbound requests.

---

## Goals

1. Define and persist `ToolDefinition` records for the Citadel query and annotate endpoints.
2. Implement `ToolCredential` storage: org-scoped, encrypted at rest, Citadel key format compliant.
3. Wire Citadel credential injection into Pantheon's outbound HTTP execution path.
4. Expose CRUD API routes for `ToolDefinition` and `ToolCredential` management.
5. Establish the pattern for future app registrations (reusable, not Citadel-specific).

---

## Non-Goals

- Implementing the `citadel_query_logs` or `citadel_annotate_log` tool logic (those are separate features in this milestone).
- Building the Citadel webhook handler (Tier 1 — separate feature `citadel-webhook-handler`).
- Frontend UI for credential management.
- Multi-tool approval gates (can be added in a follow-on).

---

## Design Overview

### Data Model

**AppRegistration** (new top-level entity)
```
id              UUID
org_slug        string       — scopes the app to one org
name            string       — human-readable (e.g. "Citadel")
base_url        string       — e.g. "https://citadel-staging.orchard9.ai"
auth_scheme     enum         — api_key | bearer_jwt | hmac
auth_header     string       — header name (e.g. "Authorization" or "X-Api-Key")
created_at      timestamp
```

**ToolDefinition** (per tool in an app)
```
id              UUID
app_id          FK → AppRegistration
name            string       — e.g. "citadel_query_logs"
description     string       — shown to agents
method          enum         — GET | POST | PATCH | DELETE
path_template   string       — e.g. "/api/v1/query"
input_schema    JSONB        — JSON Schema of the tool's parameters
output_schema   JSONB        — optional, for agent response parsing hints
requires_approval bool       — default false; true for destructive calls
created_at      timestamp
```

**ToolCredential** (per-org secret for an app)
```
id              UUID
app_id          FK → AppRegistration
org_slug        string
encrypted_key   bytes        — AES-256-GCM encrypted with org-derived key
key_prefix      string       — first 8 chars of plaintext (e.g. "ck_prod_") for UI display
created_at      timestamp
rotated_at      timestamp    — nullable
```

### Citadel API Key Format

Citadel keys follow `ck_<env>_<org>_<random>`:
- `env`: one of `prod`, `staging`, `dev`
- `org`: org identifier (alphanumeric, max 16 chars)
- `random`: 32 random hex characters

Validation regex: `^ck_(prod|staging|dev)_[a-z0-9]{1,16}_[0-9a-f]{32}$`

### Credential Encryption

Keys are encrypted with AES-256-GCM using a per-org derived encryption key:
`kdf(master_secret, org_slug)` — HKDF-SHA256, 32-byte output.
The master secret is an environment variable `PANTHEON_CREDENTIAL_KEY` (32 random bytes, base64-encoded).

### HTTP Execution

When Pantheon dispatches a tool call for a Citadel tool:
1. Look up `ToolCredential` for the calling org and app.
2. Decrypt the credential.
3. Inject as `Authorization: Bearer <key>` (or `X-Api-Key: <key>` per `auth_header` in `AppRegistration`).
4. Execute the HTTP request to `base_url + path_template` with merged parameters.
5. Return response body to the agent caller.

---

## API Routes (Pantheon)

```
POST   /api/v1/organizations/{orgSlug}/apps/register
       body: { name, base_url, auth_scheme, auth_header }
       → AppRegistration

POST   /api/v1/organizations/{orgSlug}/apps/{appId}/tools
       body: { name, description, method, path_template, input_schema, output_schema?, requires_approval }
       → ToolDefinition

POST   /api/v1/organizations/{orgSlug}/apps/{appId}/credentials
       body: { api_key: "ck_prod_..." }
       → ToolCredential (returns id + key_prefix only — plaintext never returned)

GET    /api/v1/organizations/{orgSlug}/apps/
       → list of AppRegistration + tool count

GET    /api/v1/organizations/{orgSlug}/apps/{appId}/tools
       → list of ToolDefinition

DELETE /api/v1/organizations/{orgSlug}/apps/{appId}/credentials/{credId}
       → 204 No Content
```

---

## Citadel Tool Definitions (seeded by this feature)

### citadel_query_logs
```json
{
  "name": "citadel_query_logs",
  "description": "Search Citadel logs using CPL (Citadel Processing Language). Use for finding error spikes, tracing incidents, or querying recent service logs.",
  "method": "GET",
  "path_template": "/api/v1/query",
  "input_schema": {
    "type": "object",
    "required": ["query"],
    "properties": {
      "query":      { "type": "string", "description": "CPL query string (e.g. 'level:error service:auth time:1h')" },
      "time_range": { "type": "string", "description": "Time range shorthand: 1h, 6h, 24h, 7d", "default": "1h" },
      "limit":      { "type": "integer", "minimum": 1, "maximum": 1000, "default": 100 }
    }
  },
  "requires_approval": false
}
```

### citadel_annotate_log
```json
{
  "name": "citadel_annotate_log",
  "description": "Attach an annotation to a Citadel log entry. Use to link logs to incidents, mark root causes, or flag false positives.",
  "method": "POST",
  "path_template": "/api/v1/annotations",
  "input_schema": {
    "type": "object",
    "required": ["log_id", "content", "annotation_type"],
    "properties": {
      "log_id":          { "type": "string", "description": "The Citadel log entry ID to annotate" },
      "content":         { "type": "string", "description": "Annotation text — can include markdown" },
      "annotation_type": { "type": "string", "enum": ["note", "bug", "root_cause", "false_positive", "incident"] }
    }
  },
  "requires_approval": false
}
```

The `author_type: "ai_agent"` field is always injected by Pantheon's execution layer (not exposed in input schema).

---

## Acceptance Criteria

1. A POST to `/api/v1/organizations/{orgSlug}/apps/register` with Citadel metadata creates an `AppRegistration` record and returns it.
2. A POST to `/api/v1/organizations/{orgSlug}/apps/{appId}/tools` persists a `ToolDefinition` with the given JSON schema.
3. A POST to `/api/v1/organizations/{orgSlug}/apps/{appId}/credentials` with a valid `ck_*` key stores it encrypted; the plaintext key is never returned after creation.
4. The encrypted credential is decryptable and injectable by Pantheon's HTTP execution layer when executing a tool call.
5. An invalid API key format (not matching `ck_<env>_<org>_<random>`) is rejected with HTTP 422.
6. Deleting a credential removes it and subsequent tool calls for that org/app return HTTP 401 (credential not found).
7. The `citadel_query_logs` and `citadel_annotate_log` tool definitions can be seeded via the API.

---

## Implementation Notes

- This work lives in the **Pantheon** Go codebase, not in sdlc.
- Use Pantheon's existing database (Postgres-backed) for `AppRegistration`, `ToolDefinition`, and `ToolCredential` tables.
- Credential encryption must use `PANTHEON_CREDENTIAL_KEY` env var; fail startup if absent and credentials exist.
- The `author_type: "ai_agent"` injection is the execution layer's responsibility, not the tool caller's.
- Future apps (e.g. GitHub, PagerDuty) follow the same `AppRegistration` → `ToolDefinition` → `ToolCredential` pattern.

---

## Dependencies

- `citadel-webhook-handler` — parallel; no ordering dependency for this feature.
- `citadel-query-logs-tool` and `citadel-annotate-log-tool` — depend on this feature (the `ToolDefinition` records must exist before those features implement execution).
