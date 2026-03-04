# QA Plan: citadel_query_logs Agent Tool

## Scope

Verify that the `citadel_query_logs` tool correctly:
1. Accepts valid CPL queries and forwards them to Citadel
2. Returns structured log entries and episode context
3. Handles all error cases gracefully with actionable messages
4. Embeds complete CPL skill instructions in its description
5. Enforces parameter constraints (limit cap, required fields)
6. Authenticates via `ToolCredential` — no hardcoded credentials

## Test Cases

### TC-01: Happy Path — Basic CPL Query

**Input:**
```json
{ "query": "level:error service:auth", "time_range": "1h", "limit": 50 }
```
**Expected:**
- HTTP GET sent to `<CITADEL_BASE_URL>/api/v1/query?q=level%3Aerror+service%3Aauth&range=1h&limit=50`
- `Authorization: Bearer <citadel_api_key>` header present
- Response contains `entries[]`, `episode_context[]`, `total_matched` (int), `query_duration_ms` (int)
- Each entry has: `timestamp`, `level`, `service`, `message`, `trace_id`, `metadata`

### TC-02: Default Limit

**Input:**
```json
{ "query": "level:fatal", "time_range": "30m" }
```
**Expected:**
- `limit=50` used in the request (default applied)
- No error about missing `limit`

### TC-03: Limit Clamping

**Input:**
```json
{ "query": "level:error", "time_range": "24h", "limit": 9999 }
```
**Expected:**
- Request sent with `limit=500` (clamped, not 9999)
- No error returned — clamping is silent

### TC-04: Empty Results

**Input:**
```json
{ "query": "level:fatal service:nonexistent-svc", "time_range": "1h" }
```
**Mock Response:** Citadel 200 with `{ "entries": [], "episodes": [], "total": 0, "duration_ms": 12 }`

**Expected:**
- `entries: []`
- `total_matched: 0`
- No `error` or `query_error` field
- `query_duration_ms: 12`

### TC-05: CPL Parse Error (400)

**Input:**
```json
{ "query": "lvl:error", "time_range": "1h" }
```
**Mock Response:** Citadel 400 with `{ "error": "Unknown field 'lvl'. Did you mean 'level'?" }`

**Expected:**
- Response contains `query_error: "Unknown field 'lvl'. Did you mean 'level'?"`
- `entries: []`
- `total_matched: 0`
- No crash, no stack trace exposed

### TC-06: Authentication Failure (401)

**Setup:** Configure with invalid API key
**Mock Response:** Citadel 401 `{ "error": "Unauthorized" }`

**Expected:**
- Response contains structured error: `{ "error": "...", "error_code": "auth_failed" }`
- Error message includes guidance to check `ToolCredential` / `CITADEL_API_KEY`
- No credentials leaked in error message

### TC-07: Missing Required Field — query

**Input:**
```json
{ "time_range": "1h" }
```
**Expected:**
- Validation error returned before any HTTP call
- Clear message indicating `query` is required
- No HTTP request made to Citadel

### TC-08: Missing Required Field — time_range

**Input:**
```json
{ "query": "level:error" }
```
**Expected:**
- Validation error returned before any HTTP call
- Clear message indicating `time_range` is required

### TC-09: Rate Limit (429) with Retry

**Mock Response:** Citadel 429 with `{ "error": "Rate limit exceeded", "retry_after": 1 }`

**Expected:**
- Tool waits `retry_after` seconds
- Tool retries exactly once
- If second attempt succeeds: returns normal response
- If second attempt also 429: returns error with message about rate limiting

### TC-10: Network Timeout

**Setup:** Mock server does not respond within 10s

**Expected:**
- Tool retries once
- After second timeout, returns error with timeout message
- No hang beyond ~20s total

### TC-11: Trace ID Correlation Query

**Input:**
```json
{ "query": "trace_id:abc-def-123-xyz", "time_range": "2h" }
```
**Expected:**
- Query forwarded verbatim: `?q=trace_id%3Aabc-def-123-xyz&range=2h&limit=50`
- Response entries may span multiple services (different `service` values in entries)
- No filtering applied client-side

### TC-12: Boolean Operators Preserved

**Input:**
```json
{ "query": "(level:error OR level:fatal) AND service:auth NOT service:debug", "time_range": "1h" }
```
**Expected:**
- Full CPL expression forwarded verbatim (URL-encoded) to Citadel
- No modification or simplification of boolean expression

### TC-13: Tool Name and Description

**Verification:**
- Tool `name()` returns exactly `"citadel_query_logs"`
- Tool `description()` includes all CPL basics:
  - Field filter examples: `level:error`, `service:auth`, `host:`, `trace_id:`
  - Time range examples: `1h`, `30m`, `24h`, `7d`
  - Boolean operators: `AND`, `OR`, `NOT`
  - Parentheses grouping example
  - Trace ID correlation example
  - At least 3 common incident pattern examples

### TC-14: JSON Schema Correctness

**Verification:**
- `schema()` returns valid JSON Schema
- `query` is marked as required, type string
- `time_range` is marked as required, type string
- `limit` is optional, type integer, minimum 1, maximum 500, default 50

### TC-15: No Hardcoded Credentials

**Verification (code review):**
- No API key strings in source code
- `CITADEL_BASE_URL` read from environment — not hardcoded
- `citadel_api_key` loaded from `ToolCredential` — not from env directly

## Integration Test Scenario

Run the integration test suite with a mock Citadel HTTP server:

```bash
SDLC_NO_NPM=1 cargo test citadel_query_logs --all -- --nocapture
```

All 7 tasks' test cases must pass.

## Regression Checklist

- [ ] `cargo clippy --all -- -D warnings` passes with no new warnings
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes (no regressions in other tools)
- [ ] Tool appears in tool registry listing
- [ ] Tool description visible via tool introspection command
- [ ] No `unwrap()` calls in tool implementation code

## Out of Scope

- Live Citadel API access (use mock server in CI)
- CPL validation (Citadel's responsibility)
- Discord UI rendering of results (agent's responsibility)
- Episode creation or log mutation (separate tool)
