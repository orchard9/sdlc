# Security Audit: credential-pool-core

## Scope

`crates/sdlc-server/src/credential_pool.rs` and `crates/sdlc-server/src/routes/credential_pool.rs` — PostgreSQL-backed Claude OAuth token pool.

## Threat Model

The credential pool stores long-lived Claude OAuth tokens in Postgres. Primary risks:

1. Token exfiltration via API response
2. Unauthorized write operations (add/delete/toggle)
3. SQL injection
4. Credential exhaustion / SKIP LOCKED abuse

## Findings

### [PASS] Tokens never returned over the wire

`CredentialRow` (the list response type) omits the `token` field by construction — the `list()` query selects only `id, account_name, is_active, last_used_at, use_count`. The `checkout()` call returns a `ClaudeCredential` that includes `token`, but that struct is only used internally by spawn paths — it is never serialized to an HTTP response.

**Action:** None required.

### [PASS] Write endpoints require agent token

`add_credential`, `patch_credential`, and `delete_credential` all call `require_agent_token()` before touching the pool. The helper checks the `Authorization: Bearer <token>` header against `app.agent_token` (the `SDLC_AGENT_TOKEN` value set at startup). Callers without a valid token receive a 401 before any database operation runs.

**Action:** None required.

### [PASS] No SQL injection surface

All queries use SQLx parameterized bindings (`$1`, `$2`). No string interpolation into query text. This applies to all 7 query sites.

**Action:** None required.

### [PASS] SKIP LOCKED prevents deadlocks

Concurrent `checkout()` callers each acquire a different row without blocking. A caller that finds all rows locked returns `None` immediately — no retry loop, no deadlock risk.

**Action:** None required.

### [PASS] Graceful degradation does not leak config

When `DATABASE_URL` is absent, `from_env()` logs a warning at the `warn!` level (no database URL in the log message). When the pool is `Disabled`, the status endpoint returns `"disabled — DATABASE_URL not configured"` — no database URL or credentials are exposed in the response.

**Action:** None required.

### [OBSERVE] OnceLock init race window

Between server startup and pool initialization completing, `get_status` returns `"initializing"` (200 OK, `connected: false`). Write endpoints return 500 during this window. The window is typically <100ms on a healthy Postgres. This is not a security issue but callers should retry on `"initializing"`.

**Action:** Document in API response — already handled (message field says `"initializing"`). No code change needed.

### [OBSERVE] No rate limiting on checkout

Any caller that can reach `spawn_agent_run` can drain pool credentials by spawning many concurrent agent runs. This is bounded by the pgpool `max_connections(5)` ceiling and SKIP LOCKED behavior (excess callers get `None` and fall back to ambient auth). Acceptable for the current threat model.

**Action:** Track as future hardening if pool contention becomes observable. `sdlc task add credential-pool-core "Rate-limit credential checkouts per calling pod"` deferred; not blocking.

## Verdict

**Approved.** No exploitable vulnerabilities. Token confidentiality, write authorization, and injection protections are all correctly implemented.
