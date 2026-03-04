# QA Plan: Citadel Webhook Handler in Pantheon

## Scope

Verify that `POST /api/v1/webhooks/citadel` correctly handles all documented scenarios: valid payloads create incidents, HMAC verification rejects bad signatures, severity mapping is correct, and duplicate fingerprints are handled idempotently.

## Test Environment

- Pantheon API running locally or in CI with test database
- `CITADEL_WEBHOOK_SECRET` set to a known test secret
- `CITADEL_TENANT_ORG_MAP` set to `test-tenant-uuid=test-org`
- Test org `test-org` created in the database

## Automated Test Coverage (Unit)

Run: `go test ./internal/api/handlers/... -v -run TestCitadel`

| Test | Expected |
|------|----------|
| `TestVerifyCitadelSignature/valid` | Returns `true` |
| `TestVerifyCitadelSignature/wrong_secret` | Returns `false` |
| `TestVerifyCitadelSignature/empty_header` | Returns `false` |
| `TestVerifyCitadelSignature/malformed_header` | Returns `false` |
| `TestMapCitadelSeverity/spike_any_env` | Returns `critical` |
| `TestMapCitadelSeverity/new_error_production` | Returns `high` |
| `TestMapCitadelSeverity/regression_production` | Returns `high` |
| `TestMapCitadelSeverity/threshold_production` | Returns `high` |
| `TestMapCitadelSeverity/new_error_staging` | Returns `medium` |
| `TestMapCitadelSeverity/new_error_nil_env` | Returns `medium` |
| `TestParseTenantOrgMap/valid_pairs` | Correct map |
| `TestParseTenantOrgMap/empty_string` | Empty map, no panic |
| `TestParseTenantOrgMap/whitespace` | Trimmed correctly |
| `TestHandleCitadel_ValidPayload` | 200, incident created |
| `TestHandleCitadel_InvalidSignature` | 401 |
| `TestHandleCitadel_MissingSignature` | 401 |
| `TestHandleCitadel_UnknownTenant` | 200, no incident |
| `TestHandleCitadel_DuplicateFingerprint` | 200, timeline entry added |

## Integration / Manual QA Scenarios

### QA-1: Valid payload creates incident

```bash
SECRET="test-secret-at-least-32-chars-long"
BODY='{"event_type":"new_error","timestamp":"2026-03-03T10:00:00Z","tenant_id":"test-tenant-uuid","fingerprint":"000000000000abcd","exception_type":"NullPointerException","message":"Cannot read id","language":"javascript","environment":"production","first_occurrence":"2026-03-03T09:58:00Z","dashboard_url":"https://citadel.example.com/errors/abcd"}'

SIG=$(echo -n "$BODY" | openssl dgst -sha256 -hmac "$SECRET" | awk '{print "sha256=" $2}')

curl -s -X POST http://localhost:8080/api/v1/webhooks/citadel \
  -H "Authorization: Bearer $SERVICE_KEY" \
  -H "Content-Type: application/json" \
  -H "X-Citadel-Signature: $SIG" \
  -d "$BODY"
```

Expected: `{"status":"ok","message":"...","incident_id":"<uuid>"}`

Verify in DB: incident with `alert_source = "citadel"`, `alert_name = "new_error"`, `severity = "high"`.

### QA-2: Invalid signature returns 401

Send same body with incorrect signature header.

Expected: `401 Unauthorized`

### QA-3: Missing signature returns 401

Send body without `X-Citadel-Signature` header.

Expected: `401 Unauthorized`

### QA-4: Spike event maps to critical

Same as QA-1 but with `"event_type":"spike"`.

Expected: incident with `severity = "critical"`.

### QA-5: Non-production env maps to medium

Same as QA-1 but with `"environment":"staging"`.

Expected: incident with `severity = "medium"`.

### QA-6: Unknown tenant returns 200 with no incident

Send payload with `tenant_id` not in `CITADEL_TENANT_ORG_MAP`.

Expected: `200 OK` with `{"status":"skipped",...}`. No incident in DB.

### QA-7: Duplicate fingerprint is idempotent

Send QA-1 payload twice (same fingerprint).

Expected: Second request returns `200 OK`. DB has exactly 1 incident. Second incident's timeline has a re-fire entry.

### QA-8: Malformed JSON returns 400

Send invalid JSON body with valid signature.

Expected: `400 Bad Request`.

### QA-9: Alertmanager route unaffected

Confirm `POST /api/v1/webhooks/alertmanager` still works correctly with no regression.

## Security Checklist

- [ ] Signature verification uses `crypto/subtle.ConstantTimeCompare` (no timing oracle)
- [ ] Secret is not logged at any log level
- [ ] Body is not logged before signature verification passes (prevents log injection)
- [ ] `CitadelWebhookSecret` is empty-checked; if empty, handler returns 500 with a safe error (prevents silent auth bypass)

## Pass Criteria

- All unit tests pass: `go test ./... -count=1`
- All `go vet ./...` checks pass
- QA-1 through QA-9 produce expected results
- Security checklist fully checked
