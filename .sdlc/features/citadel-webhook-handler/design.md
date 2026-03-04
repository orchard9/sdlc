# Design: Citadel Webhook Handler in Pantheon

## Overview

This feature is a contained addition to `internal/api/handlers/webhooks.go`. It adds one new handler method, one new payload type, HMAC signature verification, a severity mapping helper, and router registration. No new files are created — all code lands in existing files.

## Component Map

```
pantheon/
├── internal/domain/incident.go        — add AlertSourceCitadel constant
├── internal/config/pantheon.go        — add CitadelWebhookSecret + CitadelTenantOrgMap fields
├── internal/api/handlers/webhooks.go  — add CitadelWebhookPayload, HandleCitadel, helpers
├── internal/api/router.go             — register POST /api/v1/webhooks/citadel
└── cmd/pantheon/main.go               — wire CitadelWebhookSecret + CitadelTenantOrgMap into WebhookHandlerConfig
```

## Data Types

### `CitadelWebhookPayload`

Maps directly to Citadel's `NewErrorAlertPayload` struct from `crates/error-tracker/src/alerts/sender.rs`:

```go
type CitadelWebhookPayload struct {
    EventType      string  `json:"event_type"`       // "new_error" | "spike" | "regression" | "threshold"
    Timestamp      string  `json:"timestamp"`         // ISO 8601
    TenantID       string  `json:"tenant_id"`         // UUID
    Fingerprint    string  `json:"fingerprint"`       // hex string e.g. "000000000000abcd"
    ExceptionType  string  `json:"exception_type"`
    Message        string  `json:"message"`
    Language       string  `json:"language"`
    Environment    *string `json:"environment,omitempty"`
    ReleaseVersion *string `json:"release_version,omitempty"`
    FirstOccurrence string `json:"first_occurrence"`  // ISO 8601
    DashboardURL   string  `json:"dashboard_url"`
}
```

### `CitadelWebhookResponse`

```go
type CitadelWebhookResponse struct {
    Status     string `json:"status"`
    Message    string `json:"message"`
    IncidentID string `json:"incident_id,omitempty"`
}
```

## Handler Flow

```
POST /api/v1/webhooks/citadel
        │
        ├── 1. Read raw body bytes (io.ReadAll)
        │
        ├── 2. verifyCitadelSignature(secret, body, X-Citadel-Signature header)
        │       if invalid → 401 Unauthorized
        │
        ├── 3. json.Unmarshal body into CitadelWebhookPayload
        │       if error → 400 Bad Request
        │
        ├── 4. Look up org via citadelTenantOrgMap[payload.TenantID]
        │       if not found → log warning, return 200 OK (no incident)
        │
        ├── 5. Look up org by slug: orgRepo.GetBySlug(ctx, orgSlug)
        │       if not found → log warning, return 200 OK
        │
        ├── 6. handleCitadelAlert(ctx, org.ID, payload)
        │       → check for existing active incident (fingerprint + AlertSource="citadel")
        │         if exists → add timeline entry, return incidentID
        │         if new → create incident (severity from mapCitadelSeverity)
        │
        └── 7. Return 200 OK with CitadelWebhookResponse
```

## Signature Verification

```go
func verifyCitadelSignature(secret string, body []byte, header string) bool {
    if header == "" {
        return false
    }
    mac := hmac.New(sha256.New, []byte(secret))
    mac.Write(body)
    expected := "sha256=" + hex.EncodeToString(mac.Sum(nil))
    return subtle.ConstantTimeCompare([]byte(expected), []byte(header)) == 1
}
```

Uses `crypto/hmac`, `crypto/sha256`, `crypto/subtle`, `encoding/hex` — all stdlib, no new dependencies.

## Severity Mapping

```go
func mapCitadelSeverity(eventType string, environment *string) domain.Severity {
    if eventType == "spike" {
        return domain.SeverityCritical
    }
    env := ""
    if environment != nil {
        env = strings.ToLower(*environment)
    }
    if env == "production" || env == "prod" {
        return domain.SeverityHigh
    }
    return domain.SeverityMedium
}
```

## Org-Tenant Mapping

Parsed at handler construction from `CITADEL_TENANT_ORG_MAP` env var:

```
CITADEL_TENANT_ORG_MAP=550e8400-e29b-41d4-a716-446655440000=acme-corp,other-uuid=other-org
```

Stored as `map[string]string` on `WebhookHandler`. Parsing:

```go
func parseTenantOrgMap(raw string) map[string]string {
    m := make(map[string]string)
    for _, pair := range strings.Split(raw, ",") {
        parts := strings.SplitN(strings.TrimSpace(pair), "=", 2)
        if len(parts) == 2 {
            m[strings.TrimSpace(parts[0])] = strings.TrimSpace(parts[1])
        }
    }
    return m
}
```

## Changes to Existing Files

### `internal/domain/incident.go`

```go
// Add to AlertSource constants:
AlertSourceCitadel AlertSource = "citadel"
```

### `internal/config/pantheon.go`

```go
// Add to PantheonConfig:
CitadelWebhookSecret   string // CITADEL_WEBHOOK_SECRET
CitadelTenantOrgMap    string // CITADEL_TENANT_ORG_MAP

// Add to LoadPantheonConfig():
CitadelWebhookSecret:   GetEnv("CITADEL_WEBHOOK_SECRET", ""),
CitadelTenantOrgMap:    GetEnv("CITADEL_TENANT_ORG_MAP", ""),
```

### `internal/api/handlers/webhooks.go`

```go
// Add to WebhookHandlerConfig:
CitadelWebhookSecret string
CitadelTenantOrgMap  string

// Add to WebhookHandler struct:
citadelSecret    string
citadelTenantMap map[string]string  // tenantID → orgSlug

// Initialize in NewWebhookHandler:
citadelSecret:    cfg.CitadelWebhookSecret,
citadelTenantMap: parseTenantOrgMap(cfg.CitadelTenantOrgMap),
```

### `internal/api/router.go`

```go
// In the /webhooks route group, after alertmanager:
r.Post("/citadel", deps.WebhookHandler.HandleCitadel)
```

### `cmd/pantheon/main.go`

```go
// In WebhookHandlerConfig construction:
CitadelWebhookSecret: cfg.CitadelWebhookSecret,
CitadelTenantOrgMap:  cfg.CitadelTenantOrgMap,
```

## Incident Summary Format

```go
func buildCitadelSummary(payload CitadelWebhookPayload) string {
    env := "unknown"
    if payload.Environment != nil {
        env = *payload.Environment
    }
    s := fmt.Sprintf("Citadel %s alert: %s in %s environment.\n\nError: %s",
        payload.EventType, payload.ExceptionType, env, payload.Message)
    if payload.DashboardURL != "" {
        s += fmt.Sprintf("\n\nDashboard: %s", payload.DashboardURL)
    }
    return s
}
```

## Incident Title Truncation

```go
title := fmt.Sprintf("[Citadel] %s: %s", payload.ExceptionType, payload.Message)
if len(title) > 200 {
    title = title[:197] + "..."
}
```

## Error Handling

- Signature missing → `401` (no log of body — prevents log injection)
- Signature invalid → `401` (no log of body)
- Body parse error → `400` with error message
- Org not found for tenant → `200` with `status: "skipped"`, warning logged
- DB error on incident lookup/create → `500`, error logged, `status: "error"` in response if possible

## Testing Approach

Unit tests in `internal/api/handlers/webhooks_test.go` (or new `webhooks_citadel_test.go`):

1. `TestVerifyCitadelSignature` — valid, invalid, empty header cases
2. `TestMapCitadelSeverity` — spike=critical, new_error+production=high, new_error+staging=medium, no env=medium
3. `TestParseTenantOrgMap` — happy path, empty string, malformed pairs
4. `TestHandleCitadel_ValidPayload` — end-to-end handler test with mock repos
5. `TestHandleCitadel_InvalidSignature` — verify 401
6. `TestHandleCitadel_UnknownTenant` — verify 200 with no incident

## No New Dependencies

All crypto primitives (`crypto/hmac`, `crypto/sha256`, `crypto/subtle`, `encoding/hex`) are Go stdlib. No new `go.mod` entries required.
