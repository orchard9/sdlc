# Security Audit: Webhook Query Infrastructure

## Surface Area

This feature adds authentication (secret_token), storage mutation (store_only), and a new read endpoint (`GET /api/webhooks/{route}/data`). All three touch the webhook ingestion path on a production HTTP server.

## Findings

### F1 — Secret comparison is not constant-time [ACCEPTED]
**File**: `crates/sdlc-server/src/routes/webhooks.rs`
**Detail**: The secret header is compared with `!=` (byte-level string equality). This is not constant-time and could theoretically allow timing-based token enumeration.
**Risk**: Low. The token is a shared bearer secret (not a MAC), network jitter dwarfs any timing difference, and the endpoint is not exposed publicly without tunnel auth. This is not a meaningful attack surface at current scale.
**Action**: Accept. Documented. A future hardening pass could switch to `subtle::ConstantTimeEq` if the threat model changes.

### F2 — No rate limiting on failed secret attempts [ACCEPTED]
**File**: `crates/sdlc-server/src/routes/webhooks.rs`
**Detail**: There is no backoff or request throttling on 401 responses. A caller could brute-force the secret token.
**Risk**: Low. The token space is large (caller-controlled string), tunnel auth gates the server in cluster mode, and local mode has no external exposure.
**Action**: Accept. Consistent with the rest of the server's auth model (no global rate limiting). Track as known limitation.

### F3 — `unwrap_or(serde_json::Value::Null)` on body deserialization [OK]
**File**: `crates/sdlc-server/src/routes/webhooks.rs:206`
**Detail**: Raw body bytes that fail JSON deserialization return `Value::Null` in the query response rather than an error. This is intentional — payloads may be non-JSON (form data, binary). The raw bytes are always stored correctly; the JSON coercion is presentation-only.
**Action**: No issue. Pattern is correct.

### F4 — `unwrap()` in test code [OK]
**File**: `crates/sdlc-core/src/orchestrator/db.rs` (tests only)
**Detail**: `unwrap()` appears extensively in test functions. All are in `#[cfg(test)]` scope, not library code.
**Action**: No issue. Consistent with project test conventions.

### F5 — No auth on GET /api/webhooks/{route}/data [ACCEPTED]
**File**: `crates/sdlc-server/src/routes/webhooks.rs`, `lib.rs:615`
**Detail**: The query endpoint does not require the caller to present the route's `secret_token`. Any client with access to the server can query stored payloads.
**Risk**: Low in cluster mode (tunnel auth gates the whole server). In local mode, only localhost has access.
**Action**: Accept. Consistent with the rest of the API's auth model. A future feature could add per-route read tokens.

## Positive Findings

- All error paths in production code use `?` — no panics in library or server code.
- Payload is always stored before the `store_only` dispatch check — no data loss risk.
- `serde(default)` on new fields ensures backward-compatible deserialization of existing route records.
- Postgres migration uses `ADD COLUMN IF NOT EXISTS` — safe to run on an existing database.
- Limit parameter is capped at 10,000 (`params.limit.unwrap_or(1000).min(10000)`) — prevents unbounded result sets.

## Verdict

No blocking findings. All issues accepted with documented rationale. Feature is safe to ship.
