# Tasks: citadel_query_logs Agent Tool

## T1 — Define types: LogEntry, EpisodeContext, QueryRequest, QueryResponse

Create `types.rs` (or equivalent module) with all data types needed for the tool:
- `LogEntry { timestamp, level, service, message, trace_id, metadata }`
- `EpisodeContext { id, title, severity, started_at }`
- `QueryRequest { query, time_range, limit }` — with validation logic (required fields, limit cap)
- `QueryResponse { entries, episode_context, total_matched, query_duration_ms }`
- `ToolErrorResponse { error, error_code }` for auth/network errors
- `QueryErrorResponse { query_error, entries, total_matched }` for CPL parse errors

All types must derive `Serialize`, `Deserialize`, `Debug`.

## T2 — Implement Citadel HTTP client

Create `client.rs` with `CitadelClient`:
- Constructor takes `base_url: String` and `api_key: String`
- `query(&self, req: &QueryRequest) -> Result<QueryResponse, ClientError>`
- URL-encodes CPL query string for `?q=` param
- Sets `Authorization: Bearer <api_key>` header
- 10s timeout, 1 retry for 429 (respect `retry_after`) and network errors
- Maps Citadel response fields: `entries` → `LogEntry[]`, `episodes` → `EpisodeContext[]`, `total` → `total_matched`, `duration_ms` → `query_duration_ms`
- Maps HTTP 400 → `QueryErrorResponse`, 401 → `ToolErrorResponse { error_code: "auth_failed" }`

## T3 — Implement CitadelQueryLogsTool handler

Create `mod.rs` (or `citadel_query_logs.rs`) with `CitadelQueryLogsTool`:
- Implements `ToolHandler` trait: `name()`, `description()`, `schema()`, `invoke()`
- `name()` returns `"citadel_query_logs"`
- `description()` returns embedded CPL skill instructions string (CPL quick reference, common patterns, parameter docs)
- `schema()` returns JSON Schema with `query` (string, required), `time_range` (string, required), `limit` (integer, optional, default 50, max 500)
- `invoke()` deserializes params, runs validation, calls `CitadelClient::query`, returns result as `serde_json::Value`
- Reads `CITADEL_BASE_URL` from environment
- Loads `citadel_api_key` from `ToolCredential`

## T4 — Register tool in Pantheon tool registry

Add `CitadelQueryLogsTool` to the tool registry so agents can discover and invoke it:
- Add to `register_tools()` or equivalent bootstrap function
- Ensure the tool appears in tool listing (`sdlc tool list` or equivalent)
- Verify tool description includes the full CPL skill instructions

## T5 — Unit tests: param validation

Write unit tests for `QueryRequest` validation:
- Missing `query` field → validation error
- Missing `time_range` field → validation error
- `limit` > 500 → clamped to 500
- `limit` = 0 → defaulted to 50 or validation error
- Valid params pass through unchanged

## T6 — Unit tests: response mapping

Write unit tests for Citadel response → tool response mapping:
- Happy path: Citadel 200 with entries, episodes, total, duration_ms → correct `QueryResponse`
- Empty entries: Citadel 200 with empty `entries` array → `total_matched: 0`, no error
- CPL error: Citadel 400 with error message → `QueryErrorResponse` with `query_error` field
- Auth error: Citadel 401 → `ToolErrorResponse` with `error_code: "auth_failed"` and guidance message
- Rate limit: Citadel 429 → retry logic triggers, then surface error if still failing

## T7 — Integration test with mock Citadel server

Write an integration test that starts a mock HTTP server:
- Mock `GET /api/v1/query` returning fixture JSON
- Assert tool correctly passes `q`, `range`, `limit` params
- Assert tool correctly sets `Authorization: Bearer <key>` header
- Assert response maps correctly to tool output format
- Assert CPL error case (mock returns 400) surfaces as `query_error`
