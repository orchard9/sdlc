# Tasks: Citadel Webhook Handler in Pantheon

## T1 — Add `AlertSourceCitadel` constant to `internal/domain/incident.go`

Add `AlertSourceCitadel AlertSource = "citadel"` to the `AlertSource` const block.

**File:** `internal/domain/incident.go`
**Effort:** Trivial (1 line)

## T2 — Add Citadel config fields to `PantheonConfig`

Add `CitadelWebhookSecret` and `CitadelTenantOrgMap` to `PantheonConfig` struct and load them in `LoadPantheonConfig()`.

**File:** `internal/config/pantheon.go`
**Effort:** Small (4 lines)

## T3 — Implement `CitadelWebhookPayload`, helpers, and `HandleCitadel` in `webhooks.go`

Add to `internal/api/handlers/webhooks.go`:
- `CitadelWebhookPayload` struct
- `CitadelWebhookResponse` struct
- `citadelTenantMap map[string]string` field on `WebhookHandler`
- `citadelSecret string` field on `WebhookHandler`
- `parseTenantOrgMap(raw string) map[string]string`
- `verifyCitadelSignature(secret string, body []byte, header string) bool`
- `mapCitadelSeverity(eventType string, environment *string) domain.Severity`
- `buildCitadelSummary(payload CitadelWebhookPayload) string`
- `handleCitadelAlert(ctx, orgID string, payload CitadelWebhookPayload) (string, error)`
- `HandleCitadel(w http.ResponseWriter, r *http.Request)` — the HTTP handler
- Wire `CitadelWebhookSecret` and `CitadelTenantOrgMap` into `NewWebhookHandler`

**File:** `internal/api/handlers/webhooks.go`
**Effort:** Medium (~120 lines)
**Dependencies:** T1, T2

## T4 — Register route in router

Add `r.Post("/citadel", deps.WebhookHandler.HandleCitadel)` in the `/webhooks` route group.

**File:** `internal/api/router.go`
**Effort:** Trivial (1 line)
**Dependencies:** T3

## T5 — Wire config in `cmd/pantheon/main.go`

Pass `CitadelWebhookSecret` and `CitadelTenantOrgMap` when constructing `WebhookHandlerConfig`.

**File:** `cmd/pantheon/main.go`
**Effort:** Small (2 lines)
**Dependencies:** T2, T3

## T6 — Write unit tests

Add tests for:
- `TestVerifyCitadelSignature` — valid HMAC, invalid HMAC, empty header, wrong secret
- `TestMapCitadelSeverity` — spike=critical, new_error+production=high, new_error+staging=medium, nil env=medium, regression+production=high, threshold+nil=medium
- `TestParseTenantOrgMap` — happy path, empty string, malformed pairs, whitespace trimming
- `TestHandleCitadel_ValidPayload` — handler integration test with mock repos, valid signature, known tenant, incident created
- `TestHandleCitadel_InvalidSignature` — verify 401
- `TestHandleCitadel_MissingSignature` — verify 401
- `TestHandleCitadel_UnknownTenant` — verify 200, no incident created

**File:** `internal/api/handlers/webhooks_citadel_test.go` (new test file)
**Effort:** Medium (~100 lines)
**Dependencies:** T3

## T7 — Verify `go test ./...` and `go vet ./...` pass

Run the full Go test suite and vet to confirm no regressions.

**Dependencies:** T1–T6
