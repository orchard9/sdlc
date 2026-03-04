# Design: citadel_query_logs Agent Tool

## Architecture Overview

This is a backend/CLI tool — no UI is needed. The tool integrates as a Pantheon agent tool definition that agents can invoke from Discord or other orchestrated contexts.

```
Discord Agent
     │
     ▼
Pantheon Tool Registry
     │ invoke("citadel_query_logs", { query, time_range, limit })
     ▼
citadel_query_logs Tool Handler
     │
     ├── Validate params (limit cap, required fields)
     ├── Load ToolCredential → extract citadel_api_key
     ├── Read CITADEL_BASE_URL from env/config
     │
     ▼
GET /api/v1/query?q=<CPL>&range=<time_range>&limit=<limit>
Authorization: Bearer <citadel_api_key>
     │
     ▼
Citadel API Response
     │
     ├── Map log entries → { timestamp, level, service, message, trace_id, metadata }
     ├── Extract episode_context from response
     ├── Capture total_matched, query_duration_ms
     │
     ▼
Tool Response JSON → Agent
```

## Module Structure

```
crates/sdlc-core/src/tools/citadel_query_logs/
├── mod.rs          — Tool definition, registration, ToolHandler impl
├── client.rs       — HTTP client for Citadel /api/v1/query
├── types.rs        — Request/response types (LogEntry, EpisodeContext, QueryResponse)
└── skill.md        — Embedded skill instructions (CPL basics)
```

Or, if using a single-file approach for smaller tools:
```
crates/sdlc-core/src/tools/citadel_query_logs.rs  — all of the above in one file
```

## Tool Definition

```rust
pub struct CitadelQueryLogsTool {
    credential: ToolCredential,
    base_url: String,
}

impl ToolHandler for CitadelQueryLogsTool {
    fn name(&self) -> &str { "citadel_query_logs" }
    fn description(&self) -> &str { SKILL_INSTRUCTIONS }
    fn schema(&self) -> serde_json::Value { /* JSON Schema for params */ }
    fn invoke(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError>;
}
```

## Data Types

### Tool Input
```json
{
  "query": "level:error service:auth",
  "time_range": "1h",
  "limit": 50
}
```

### Tool Output (success)
```json
{
  "entries": [
    {
      "timestamp": "2026-03-03T19:00:00Z",
      "level": "error",
      "service": "auth",
      "message": "Token validation failed for user 12345",
      "trace_id": "abc-def-123",
      "metadata": { "host": "api-01", "region": "us-east-1" }
    }
  ],
  "episode_context": [
    {
      "id": "ep-001",
      "title": "Auth service degradation",
      "severity": "high",
      "started_at": "2026-03-03T18:45:00Z"
    }
  ],
  "total_matched": 142,
  "query_duration_ms": 87
}
```

### Tool Output (query error)
```json
{
  "query_error": "Unknown field 'lvl' in CPL expression. Did you mean 'level'?",
  "entries": [],
  "total_matched": 0
}
```

### Tool Output (auth error)
```json
{
  "error": "Authentication failed. Check that CITADEL_API_KEY is set in ToolCredential.",
  "error_code": "auth_failed"
}
```

## Citadel API Contract

```
GET /api/v1/query
Authorization: Bearer <api_key>
Query params:
  q       = <CPL string, URL-encoded>
  range   = <time range, e.g. "1h">
  limit   = <integer, 1-500>

Response 200:
{
  "entries": [...],
  "episodes": [...],
  "total": <int>,
  "duration_ms": <int>
}

Response 400: { "error": "<CPL parse error message>" }
Response 401: { "error": "Unauthorized" }
Response 429: { "error": "Rate limit exceeded", "retry_after": <seconds> }
```

## Skill Instructions (Embedded in Tool Description)

The tool's `description()` field includes the following CPL quick-reference so agents can write effective queries without external documentation:

```
citadel_query_logs — Search Citadel logs using CPL (Citadel Processing Language).

CPL QUICK REFERENCE:
  Field filters:   level:error  service:auth  host:api-01  trace_id:abc123
  Time ranges:     1h  30m  24h  7d  (passed as time_range param)
  Boolean:         level:error AND service:auth
                   level:error OR level:fatal
                   level:error NOT service:debug-svc
                   (level:error OR level:fatal) AND service:payment
  Trace correlation: trace_id:abc-def-123  (fetches all services sharing that trace)

COMMON PATTERNS:
  All errors last hour:         level:error time:1h
  Fatals in specific service:   level:fatal service:payment time:30m
  Trace investigation:          trace_id:<id> time:2h
  Cross-service errors:         (level:error OR level:fatal) service:auth OR service:gateway time:1h

PARAMETERS:
  query      (required) CPL expression string
  time_range (required) Relative range: 1h, 30m, 24h, 7d
  limit      (optional) Max entries returned, default 50, max 500
```

## Error Handling Strategy

| Scenario | Behavior |
|----------|----------|
| CPL parse error (400) | Return `query_error` field with Citadel's message |
| Auth failure (401) | Return structured error with guidance to check ToolCredential |
| Rate limit (429) | Retry once after `retry_after` seconds, then surface error |
| Network timeout | Single retry, then surface timeout error |
| Empty results | Return `entries: []`, `total_matched: 0` — not an error |
| `limit` > 500 | Clamp to 500 silently |
| Missing `query` | Return validation error before calling API |

## Configuration

- `CITADEL_BASE_URL` — environment variable for Citadel API base URL (required)
- `citadel_api_key` — from `ToolCredential` store (required)
- Timeout: 10s per HTTP call, 1 retry for transient failures

## Sequence: Happy Path

```
1. Agent invokes tool with { query: "level:error service:auth", time_range: "1h" }
2. Tool validates params, defaults limit=50
3. Tool loads ToolCredential → citadel_api_key
4. Tool reads CITADEL_BASE_URL
5. HTTP GET /api/v1/query?q=level%3Aerror+service%3Aauth&range=1h&limit=50
   Authorization: Bearer <key>
6. Citadel responds 200 with entries, episodes, total, duration_ms
7. Tool maps response → LogEntry[], EpisodeContext[]
8. Tool returns { entries, episode_context, total_matched, query_duration_ms }
9. Agent reads entries and responds in Discord with incident summary
```

## Testing Approach

- Unit tests: param validation (missing query, limit clamping)
- Unit tests: response mapping (Citadel JSON → tool response)
- Unit tests: error mapping (400 → query_error, 401 → auth error)
- Integration test: mock Citadel HTTP server returning fixture responses
- Skill instruction test: validate all CPL example patterns parse correctly against Citadel in staging
