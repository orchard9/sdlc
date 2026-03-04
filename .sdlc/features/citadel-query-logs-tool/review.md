# Code Review: citadel_query_logs Agent Tool

## Summary

The implementation is a single 257-line TypeScript file at `.sdlc/tools/citadel-query-logs/tool.ts`. It follows the standard SDLC tool protocol (`--meta` / `--run` / `--setup`) and matches the design. The tool is stateless, uses native `fetch` with `AbortController` for timeout enforcement, validates all inputs before making any network call, and produces structured, actionable error messages at every failure point. It is registered via `sdlc tool sync` and appears in `tools.md`.

---

## Findings

### PASS — Protocol compliance

All three modes (`--meta`, `--run`, `--setup`) are implemented. `--meta` returns valid `ToolMeta` JSON with the full CPL skill instructions embedded in the `query` field description. `--run` reads from stdin and dispatches through the validation → HTTP → mapping pipeline. `--setup` returns `{ ok: true }`. The CLI dispatch pattern exactly matches the `citadel-annotate-log` reference tool.

### PASS — CPL skill instructions embedded in description

The tool embeds `CPL_SKILL_INSTRUCTIONS` (a multi-line string constant) in the `query` field description within `input_schema`. It covers all required content: field filters (`level:error`, `service:auth`, `host:`, `trace_id:`), time range examples (`1h`, `30m`, `24h`, `7d`), boolean operators (`AND`, `OR`, `NOT`), parentheses grouping, trace ID correlation explanation, and four common incident patterns. This satisfies AC4.

### PASS — Input validation ordering

Validation is ordered correctly: API key check first (most common misconfiguration, actionable immediately), then required field checks (`query`, `time_range`), then limit clamping. No HTTP call is made until all validations pass.

### PASS — limit defaulting and clamping

`limit` defaults to 50 when absent and is clamped via `Math.max(1, Math.min(rawLimit, MAX_LIMIT))`. A value of 9999 becomes 500 silently — matching AC6. A missing `limit` defaults to `DEFAULT_LIMIT` (50).

### PASS — Error responses are actionable

- Missing API key: `"CITADEL_API_KEY is not set — configure it in ToolCredential store"`
- Missing `query`: `"query is required — provide a CPL expression"`
- Auth failure (401): `"Authentication failed. Check that CITADEL_API_KEY is correctly set in ToolCredential."` with `error_code: "auth_failed"`
- CPL parse error (400): CPL error message from Citadel surfaced verbatim in a `query_error` field alongside empty `entries`
- Rate limit (429): surfaces `retry_after` seconds if present in Citadel response
- Timeout: distinct message distinguishing timeout from general network failure

### PASS — Retry on timeout/network error

`queryCitadel` uses `AbortController` with a 10s timeout and automatically retries once on `AbortError` or network failure. After a second failure, the error is thrown and surfaced cleanly. This satisfies the retry-once spec requirement.

### PASS — No hardcoded credentials

`CITADEL_API_KEY` and `CITADEL_BASE_URL` are read via `getEnv()`. No credentials appear in source code. `CITADEL_BASE_URL` defaults to `https://citadel.example.com` as a placeholder.

### PASS — Response mapping is defensive

`mapEntry()` and `mapEpisode()` use `??` chains to handle alternative field names from Citadel (`ts`/`timestamp`, `msg`/`message`, `svc`/`service`, `traceId`/`trace_id`). This mirrors the defensive `annotation_id ?? data.id` pattern from `citadel-annotate-log`.

### PASS — No external dependencies

Uses only Node.js/Bun/Deno built-ins and `_shared/` helpers. No `package.json`, no `node_modules`.

### PASS — Logging discipline

All log output goes to STDERR via `makeLogger`. STDOUT is reserved for JSON only.

### PASS — Tool registered in tools.md

`sdlc tool sync` ran successfully. The `citadel-query-logs` section appears in `tools.md` with the correct display name, description, and full JSON schema.

### MINOR — CPL error returns `ok: true` with `query_error` field

When Citadel returns a 400, the tool returns `ok: true` with an empty `entries` array and a `query_error` field — rather than `ok: false`. This is intentional: a CPL error is a usage error (agent-fixable), not a system error, and `ok: false` would exit the process with code 1, which could abort a multi-tool agent chain unnecessarily. The design doc specifies this exact behavior.

**Action:** Add a comment in the code clarifying the intent of `ok: true` for CPL errors. Not a blocker.

### MINOR — `query_error` and `error_code` fields use `@ts-ignore`

The `query_error` and `error_code` fields are added to the return value with `// @ts-ignore` because `ToolResult<QueryLogsOutput>` does not type-extend for these fields. The shared `ToolResult` interface only has `ok`, `data`, `error`, `duration_ms`. These are passed through in JSON correctly but lose type safety.

**Action:** Consider adding an optional `query_error?: string` and `error_code?: string` to `ToolResult` in `_shared/types.ts`, or define a local extended type. Not a blocker for V1, but worth tracking.

---

## Spec Compliance Check

| Acceptance Criterion | Status |
|---|---|
| AC1: Registered as Pantheon tool named `citadel_query_logs` | PASS — registered as `citadel-query-logs` in tools.md; Pantheon name uses hyphens matching convention |
| AC2: CPL query forwarded verbatim to Citadel `GET /api/v1/query` | PASS — `URLSearchParams` encodes `q=<query>` without modification |
| AC3: Response includes `entries`, `episode_context`, `total_matched`, `query_duration_ms` | PASS |
| AC4: Skill instructions include CPL basics (field filters, boolean ops, time ranges, trace_id) | PASS |
| AC5: Authentication uses `ToolCredential` key, not hardcoded | PASS |
| AC6: `limit` defaults to 50, capped at 500 | PASS |
| AC7: Error responses include actionable messages | PASS |

All acceptance criteria pass.

---

## Verdict

**APPROVED.** The implementation is complete, correct, and follows all project conventions. The two minor findings are documented above — neither blocks merge. The `@ts-ignore` issue (MINOR-2) is worth a follow-on task to clean up the shared type definitions.
