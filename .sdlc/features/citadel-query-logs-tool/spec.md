# Spec: citadel_query_logs Agent Tool

## Overview

Implement the `citadel_query_logs` Pantheon agent tool, enabling Discord-invoked agents to query Citadel logs using CPL (Citadel Processing Language). This tool empowers on-call agents to fetch logs around incidents without leaving the Discord workflow.

## Problem Statement

When incidents occur, agents responding via Discord need rapid access to log data from Citadel. Currently, querying logs requires direct API access and knowledge of CPL syntax — neither of which is readily available in a Discord context. This tool bridges the gap, wrapping Citadel's query API in an agent-accessible interface.

## Goals

- Provide a Pantheon agent tool that executes CPL queries against Citadel's log storage
- Return structured log entries with episode context to help agents understand the incident
- Include skill instructions with CPL basics so agents can write effective queries without documentation lookups
- Authenticate using the stored Citadel API key from `ToolCredential`

## Non-Goals

- Does not implement CPL parsing or validation (delegated to Citadel)
- Does not write or mutate log data
- Does not create Citadel episodes (see `citadel-annotate-log-tool`)

## Tool Parameters

| Parameter    | Type   | Required | Description |
|-------------|--------|----------|-------------|
| `query`     | string | yes      | CPL query expression (e.g. `level:error service:auth`) |
| `time_range`| string | yes      | Relative time range (e.g. `1h`, `30m`, `24h`) |
| `limit`     | int    | no       | Maximum log entries to return (default: 50, max: 500) |

## Tool Response

Returns a JSON object with:
- `entries`: array of log entries, each containing `timestamp`, `level`, `service`, `message`, `trace_id`, `metadata`
- `episode_context`: any Citadel episodes (incidents, annotations) associated with the queried time range and services
- `total_matched`: total count of matching entries (may exceed `limit`)
- `query_duration_ms`: how long the query took

## API Integration

- **Endpoint**: `GET /api/v1/query`
- **Auth**: `Authorization: Bearer <citadel_api_key>` from `ToolCredential`
- **Citadel base URL**: from project config or environment variable `CITADEL_BASE_URL`
- **Parameters**: `q` (CPL string), `range` (time range), `limit`

## Skill Instructions — CPL Basics

The skill instructions embedded in this tool must include CPL fundamentals:

### Field Filters
- `level:error` — filter by log level (debug, info, warn, error, fatal)
- `service:auth` — filter by service name
- `host:api-01` — filter by host
- `trace_id:abc123` — correlate across services by trace ID

### Time Range Examples
- `1h` — last 1 hour
- `30m` — last 30 minutes
- `24h` — last 24 hours
- `7d` — last 7 days

### Boolean Operators
- `level:error AND service:auth` — AND (both conditions)
- `level:error OR level:fatal` — OR (either condition)
- `level:error NOT service:debug-svc` — NOT (exclude)
- Parentheses for grouping: `(level:error OR level:fatal) AND service:auth`

### Trace ID Correlation
When investigating an incident with a known trace ID:
```
trace_id:abc-def-123 time:1h
```
This fetches all log lines across all services that share the trace, showing the full request journey.

### Common Incident Patterns
- Fetch errors in auth service last hour: `level:error service:auth time:1h`
- Fetch fatal errors across all services: `level:fatal time:30m`
- Correlate a specific trace: `trace_id:<id> time:2h`

## Error Handling

- Invalid CPL syntax → return Citadel's error message verbatim with `query_error` field
- Authentication failure → surface as tool error with guidance to check `CITADEL_API_KEY`
- Network timeout → retry once, then surface timeout error
- Empty results → return empty `entries` array with `total_matched: 0` (not an error)

## Acceptance Criteria

1. Tool is registered as a Pantheon agent tool under the name `citadel_query_logs`
2. CPL query is forwarded verbatim to Citadel `GET /api/v1/query`
3. Response includes `entries`, `episode_context`, `total_matched`, `query_duration_ms`
4. Skill instructions include CPL basics covering field filters, boolean operators, time ranges, and trace_id correlation
5. Authentication uses `ToolCredential` key, not hardcoded credentials
6. `limit` defaults to 50 and is capped at 500
7. Error responses include actionable error messages
