# Spec: Citadel Webhook Handler in Pantheon

## Problem

Citadel's error-tracker sends HMAC-SHA256-signed webhook notifications to configured URLs when error thresholds are crossed (new errors, spikes, regressions, count thresholds). Pantheon has no receiver for these webhooks — there is no route, no HMAC verification, and no mapping from Citadel error levels to Pantheon incident severity. Teams must manually create Pantheon incidents when Citadel fires, introducing latency in incident response.

## Goal

Add `POST /api/v1/webhooks/citadel` to Pantheon. This endpoint:
1. Accepts HMAC-SHA256-signed error alert payloads from Citadel.
2. Verifies the signature using a per-org shared secret stored in Pantheon config.
3. Maps Citadel error level (`critical`, `error`, `warn`, `info`) and alert type (`new_error`, `spike`, `regression`, `threshold`) to Pantheon `Severity`.
4. Creates a Pantheon incident automatically, following the same pattern as `HandleAlertmanager`.
5. Uses `AlertSource = "citadel"` on created incidents.

## Citadel Webhook Payload

Citadel sends the following JSON body via `POST` to the configured webhook URL. The request includes an `X-Citadel-Signature` header with value `sha256=<hex-hmac>` computed over the raw body bytes using the tenant's `signing_secret`.

```json
{
  "event_type": "new_error",
  "timestamp": "2026-03-03T10:00:00Z",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "fingerprint": "000000000000abcd",
  "exception_type": "NullPointerException",
  "message": "Cannot read property 'id' of undefined",
  "language": "javascript",
  "environment": "production",
  "release_version": "v2.3.1",
  "first_occurrence": "2026-03-03T09:58:00Z",
  "dashboard_url": "https://citadel.example.com/errors/000000000000abcd"
}
```

`event_type` values: `new_error`, `spike`, `regression`, `threshold`.

The `X-Citadel-Signature` header format matches Citadel's signing pattern from `citadel-auth/src/webhooks.rs`:
```
X-Citadel-Signature: sha256=<lowercase-hex-hmac-sha256>
```

## Severity Mapping

Citadel does not embed a direct severity level in the payload; severity is inferred from `event_type` and `environment`:

| Citadel event_type | environment   | Pantheon Severity |
|--------------------|---------------|-------------------|
| `spike`            | any           | `critical`        |
| `new_error`        | `production`  | `high`            |
| `regression`       | `production`  | `high`            |
| `threshold`        | `production`  | `high`            |
| `new_error`        | non-prod/none | `medium`          |
| `regression`       | non-prod/none | `medium`          |
| `threshold`        | non-prod/none | `medium`          |

This mapping mirrors the approach in `mapAlertSeverity` for Alertmanager: explicit, table-driven, no hidden heuristics.

## Authentication

The Citadel webhook uses HMAC-SHA256 signature verification, **not** the existing `ServiceKeyMiddleware` Bearer token. The route is mounted under the existing `/api/v1/webhooks` group (which requires service key middleware), but verification adds a second layer: the handler validates `X-Citadel-Signature` against the raw request body using a `CITADEL_WEBHOOK_SECRET` environment variable.

If the signature is absent or invalid, the handler returns `401 Unauthorized`.

This matches Citadel's own `verify_signature` logic in `citadel-auth/src/webhooks.rs`:
```
expected = "sha256=" + hex(hmac_sha256(secret, raw_body))
constant_time_compare(expected, header_value)
```

## Configuration

One new environment variable:

| Variable                 | Description                                           |
|--------------------------|-------------------------------------------------------|
| `CITADEL_WEBHOOK_SECRET` | Shared signing secret (min 32 chars, matches Citadel's `signing_secret` field in `webhook_configs` table) |

Loaded in `PantheonConfig` alongside existing `ServiceKey`.

## Org Routing

Citadel payloads include `tenant_id` (UUID). Pantheon must resolve this to an org. For the initial implementation, the mapping is provided via the `CITADEL_TENANT_ORG_MAP` environment variable (see Org-Tenant Mapping below). If no org is found for a given tenant ID, the webhook is accepted (returns `200 OK`) but no incident is created — the event is logged with a warning. A DB-backed mapping can follow as a subsequent feature.

## Incident Creation

When a matching org is found:
1. Check for an existing active incident with `AlertFingerprint = fingerprint` and `AlertSource = "citadel"`. If found, add a timeline entry (idempotent re-fire behavior).
2. If no existing incident, create one with:
   - `Title`: `[Citadel] <exception_type>: <message>` (truncated to 200 chars)
   - `Severity`: derived from the mapping table above
   - `Status`: `investigating`
   - `AlertSource`: `"citadel"` (new constant added to `domain/incident.go`)
   - `AlertFingerprint`: the hex fingerprint string
   - `AlertName`: `event_type` value
   - `AlertLabels`: `{"event_type": ..., "language": ..., "environment": ..., "release_version": ...}`
   - `AlertAnnotations`: `{"dashboard_url": ..., "exception_type": ..., "first_occurrence": ...}`
   - `Summary`: formatted summary including exception type, event type, environment, and Citadel dashboard URL
   - Timeline entry: `"Incident auto-created from Citadel alert: <event_type> — <exception_type>"`

## New Code

### `internal/domain/incident.go`

Add:
```go
AlertSourceCitadel AlertSource = "citadel"
```

### `internal/api/handlers/webhooks.go`

Add:
- `CitadelWebhookPayload` struct (mirrors `NewErrorAlertPayload` from Citadel's sender.rs)
- `CitadelWebhookResponse` struct
- `HandleCitadel(w http.ResponseWriter, r *http.Request)` method on `WebhookHandler`
- `verifyCitadelSignature(secret string, body []byte, header string) bool` helper (constant-time HMAC comparison)
- `mapCitadelSeverity(eventType, environment string) domain.Severity` helper
- `handleCitadelAlert(ctx, orgID, payload) (string, error)` — incident creation/update logic

`WebhookHandlerConfig` gains:
```go
CitadelWebhookSecret string
```

### `internal/api/router.go`

In the `/webhooks` route group:
```go
if deps.WebhookHandler != nil {
    r.Post("/alertmanager", deps.WebhookHandler.HandleAlertmanager)
    r.Post("/citadel", deps.WebhookHandler.HandleCitadel)
}
```

### `internal/config/pantheon.go`

Add to `PantheonConfig`:
```go
CitadelWebhookSecret string // CITADEL_WEBHOOK_SECRET
```

Load in `LoadPantheonConfig()`:
```go
CitadelWebhookSecret: GetEnv("CITADEL_WEBHOOK_SECRET", ""),
```

### `cmd/pantheon/main.go`

Pass `CitadelWebhookSecret` when constructing `WebhookHandlerConfig`.

## Org-Tenant Mapping (Minimal Implementation)

Rather than a new table, the initial implementation uses a configuration-level approach: `CITADEL_TENANT_ORG_MAP` environment variable as a comma-separated list of `<citadel-tenant-id>=<org-slug>` pairs. The handler parses this map at startup and uses it for routing.

Example: `CITADEL_TENANT_ORG_MAP=550e8400-e29b-41d4-a716-446655440000=acme-corp,other-uuid=other-org`

This avoids a DB migration for the initial feature while remaining functional. A proper DB-backed mapping can follow as a subsequent feature.

## REST API Response

`POST /api/v1/webhooks/citadel`

**Request headers:**
- `X-Citadel-Signature: sha256=<hex>` (required)
- `Content-Type: application/json`

**Request body:** `CitadelWebhookPayload` JSON

**Response:**
- `200 OK` with `CitadelWebhookResponse`:
  ```json
  {
    "status": "ok",
    "message": "processed citadel alert",
    "incident_id": "uuid-or-empty"
  }
  ```
- `401 Unauthorized` if signature is missing or invalid
- `400 Bad Request` if body cannot be parsed

**Note:** Always return `200` for valid, authenticated payloads even when no incident is created (e.g., unrecognized tenant). This matches Citadel's expectation and prevents unnecessary retries.

## Acceptance Criteria

1. `POST /api/v1/webhooks/citadel` with valid HMAC signature and known tenant → `200 OK`, Pantheon incident created with `AlertSource = "citadel"`.
2. Duplicate payload (same fingerprint) → `200 OK`, existing incident updated with timeline entry, no new incident created.
3. Unknown Citadel tenant ID (no org mapping) → `200 OK`, no incident created, warning logged.
4. Invalid or missing `X-Citadel-Signature` → `401 Unauthorized`.
5. Malformed JSON body → `400 Bad Request`.
6. `event_type = "spike"` → incident with `Severity = "critical"` regardless of environment.
7. `event_type = "new_error"` with `environment = "production"` → `Severity = "high"`.
8. `event_type = "new_error"` with `environment = "staging"` or absent → `Severity = "medium"`.
9. All new code passes existing Go test suite (`go test ./...`).
10. `verifyCitadelSignature` uses constant-time comparison (no timing oracle).

## Out of Scope

- UI for configuring Citadel webhook endpoints (Citadel dashboard handles this).
- DB-backed org-to-tenant-ID mapping (covered by a future feature).
- Handling Citadel auth webhook events (`auth_failed`, `key_revoked`) — those go to a separate security pipeline.
- Webhook registration API in Pantheon (operators configure Pantheon's URL in Citadel's dashboard manually).
