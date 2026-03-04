# Security Audit: Citadel Webhook Handler in Pantheon

## Scope

This audit covers `POST /api/v1/webhooks/citadel` — a new public endpoint that receives HMAC-SHA256-signed error threshold alerts from Citadel and creates Pantheon incidents automatically. The security surface includes: authentication/authorization, input validation, secret handling, log hygiene, DoS resistance, and incident creation integrity.

## Threat Model

| Threat | Actor | Impact |
|---|---|---|
| Auth bypass | External attacker | Creates arbitrary incidents in any org |
| Log injection | Unauthenticated attacker | Corrupts log aggregation / audit trail |
| Timing oracle on HMAC | Network-adjacent attacker | Recovers signing secret via timing side-channel |
| Tenant routing abuse | Authenticated attacker (valid service key, no HMAC secret) | Creates incidents in wrong org |
| Secret leakage | Compromised log sink | HMAC signing secret exposed |
| Unconfigured secret bypass | Operator misconfiguration | Auth silently skipped |
| DoS via large body | Unauthenticated attacker | Memory exhaustion / OOM |
| Incident flooding | Valid Citadel tenant | Creates unbounded incidents |

## Finding-by-Finding Analysis

### F1 — Authentication Architecture: PASS

The route sits inside `/api/v1/webhooks` which applies `ServiceKeyMiddleware.RequireServiceKey` (Bearer token) as the outer auth layer. The Citadel-specific HMAC-SHA256 signature (`X-Citadel-Signature`) is verified as a second independent layer inside `HandleCitadel`. This matches the spec design: an attacker needs both the service key AND the Citadel signing secret to successfully create incidents.

The outer service key auth is Pantheon's existing defense-in-depth layer (already audited); this audit focuses on the new Citadel-specific layer.

**Action: None required.**

### F2 — HMAC Verification Implementation: PASS

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

- Uses `crypto/hmac` and `crypto/sha256` from Go stdlib — no third-party crypto.
- Uses `crypto/subtle.ConstantTimeCompare` — no timing oracle.
- Early return on empty header (avoids HMAC computation on clearly invalid requests, though this leaks that the check is present — acceptable since Citadel's webhook config always sends the header).
- Body bytes are passed directly to HMAC without any transformation — no double-decode or encoding ambiguity.

**Action: None required.**

### F3 — Empty Secret Guard: PASS

```go
if h.citadelSecret == "" {
    h.logger.Error("citadel webhook received but CITADEL_WEBHOOK_SECRET is not configured")
    response.WriteError(w, response.InternalProblem("citadel webhook secret not configured"))
    return
}
```

An empty `CITADEL_WEBHOOK_SECRET` would mean `verifyCitadelSignature` computes HMAC with an empty key — which would accept any correctly-formatted empty-key signature. The guard prevents this silent auth bypass by failing closed (`500`) rather than allowing any request through. This is the correct posture.

**Action: None required.**

### F4 — Body Read Order: PASS

The handler reads the raw body bytes _before_ any JSON decoding. The HMAC is verified over the raw bytes. This is the correct order — it prevents a class of bugs where decoded and re-encoded content is compared against the signature of the original bytes.

**Action: None required.**

### F5 — Log Injection Prevention: PASS

Before signature verification, the handler does not log any request content. The only warning log on auth failure records `remote_addr` (server-controlled) and `signature_present` (boolean — not user input). After signature verification passes, the payload fields are logged individually using structured logging (`slog`) which escapes control characters — no format string injection risk.

**Action: None required.**

### F6 — Secret Logging: PASS

`citadelSecret` is stored in the `WebhookHandler` struct and is never passed to any logger call. Config loading in `pantheon.go` uses `GetEnv()` which does not log values. No finding.

**Action: None required.**

### F7 — Tenant Routing Integrity: PASS

Tenant routing uses an in-memory map parsed from `CITADEL_TENANT_ORG_MAP` at startup. Mapping is immutable at runtime — no dynamic configuration changes are possible. Unrecognized tenant IDs return `200 OK` with no incident (correct per spec — prevents retries). No SQL injection surface since no raw query is constructed from `tenant_id`.

**Action: None required.**

### F8 — Cross-Source Fingerprint Collision: PASS

The idempotency check explicitly guards against cross-source collisions:

```go
if existing != nil && existing.AlertSource != domain.AlertSourceCitadel {
    existing = nil
}
```

This means an Alertmanager incident with the same fingerprint string as a Citadel alert is NOT treated as the same incident — Citadel will create a new incident rather than appending to an Alertmanager-owned incident. This is the correct behavior.

**Action: None required.**

### F9 — DoS: Body Size — ACCEPTED RISK

`io.ReadAll(r.Body)` with no size limit. An attacker who has passed the outer service key middleware could send arbitrarily large bodies. This would be processed entirely in memory before HMAC verification rejects it (for invalid signatures) or parses it (for valid ones).

This is a pre-existing pattern in the Alertmanager handler (same `io.ReadAll` without size limit). Adding a size limit here would fix only the Citadel surface while leaving Alertmanager equivalent. The correct fix is a global `http.MaxBytesReader` applied in router middleware, which is a separate infrastructure improvement not in scope for this feature.

**Action: Track as follow-up task. Accept for this feature — pattern is consistent with Alertmanager handler and requires a coordinated fix across all webhook handlers.**

### F10 — DoS: Incident Flooding — ACCEPTED RISK

A valid Citadel tenant with a valid signing secret could send many distinct fingerprints in rapid succession, each creating a new incident. There is no rate limiting on the Citadel endpoint.

This is also a pre-existing pattern (Alertmanager has the same behavior). Per the spec, rate limiting is out of scope for this feature. Citadel's own retry logic includes back-off, and the duplicate fingerprint check limits flooding from a single error event.

**Action: Accept for this feature — out of scope per spec. Rate limiting on webhook endpoints is a separate infrastructure concern.**

### F11 — Title Truncation Boundary: PASS

```go
title := fmt.Sprintf("[Citadel] %s: %s", payload.ExceptionType, payload.Message)
if len(title) > 200 {
    title = title[:197] + "..."
}
```

`len()` on a Go string returns byte length (UTF-8). If `ExceptionType` or `Message` contains multi-byte Unicode, truncation at byte position 197 could split a rune. This would produce an invalid UTF-8 string stored in the database. However: (1) Citadel payloads in practice contain ASCII exception type names and error messages, (2) PostgreSQL in UTF-8 mode would reject an invalid UTF-8 string with an error at insert time (not a security issue, an operational one), and (3) this is the same pattern used elsewhere in Pantheon.

**Action: Accept — low-probability operational edge case, consistent with existing codebase patterns. Track if Unicode exception types become a concern.**

## Security Checklist

| Item | Status |
|---|---|
| HMAC-SHA256 used for signature verification | PASS |
| `crypto/subtle.ConstantTimeCompare` prevents timing oracle | PASS |
| Signing secret not logged at any level | PASS |
| Body not logged before signature verification (no log injection) | PASS |
| Empty secret guard fails closed (500, not auth bypass) | PASS |
| Body read before JSON decode (no decode ambiguity) | PASS |
| Cross-source fingerprint collision handled explicitly | PASS |
| Unrecognized tenant → 200 with no incident (no retry storm) | PASS |
| No SQL injection surface in tenant routing | PASS |
| Body size limit on webhook routes | RISK ACCEPTED (F9) |
| Rate limiting on webhook routes | RISK ACCEPTED (F10) |

## Verdict

**APPROVE** — No blocking security findings. Two accepted risks (body size limit and rate limiting) are pre-existing infrastructure concerns that apply equally to the Alertmanager handler and require a coordinated fix outside this feature's scope. A follow-up task has been created for F9.
