# Security Audit: citadel_query_logs Agent Tool

## Scope

Security review of `.sdlc/tools/citadel-query-logs/tool.ts` — the Pantheon agent tool that queries Citadel log storage via CPL from Discord.

## Threat Model

The tool is invoked by an AI agent (Discord-triggered) with user-supplied CPL query strings. The primary risks are:

1. **Credential exposure** — API key leaking in logs, errors, or responses
2. **CPL injection** — malicious query strings sent to Citadel causing unintended data access
3. **Log data leakage** — sensitive log content returned to unauthorized callers
4. **Denial of service** — unbounded requests, no timeout, or resource exhaustion
5. **SSRF** — manipulated `CITADEL_BASE_URL` redirecting requests to internal services

---

## Findings

### PASS — API key not logged or echoed

`CITADEL_API_KEY` is loaded via `getEnv('CITADEL_API_KEY')` and used only in the `Authorization` header. It is never logged (checked all `log.info` and `log.error` calls), never included in error messages, and never returned in any response field. No credential leakage path found.

### PASS — No CPL injection amplification

CPL queries are forwarded verbatim to Citadel via `URLSearchParams`, which URL-encodes the string. The tool does not evaluate, parse, or execute the CPL expression locally — Citadel is the only interpreter. Any CPL injection risk is contained to what the authenticated Citadel API key can access. If an agent constructs a malicious CPL query, it can only read data within the API key's scope — it cannot write, delete, or escalate.

**Residual risk:** A malicious caller could craft a CPL query that returns sensitive log data (e.g. `level:debug service:payments`). This is an authorization concern for the Citadel API key scope, not this tool.
**Action:** Ensure `CITADEL_API_KEY` used in production has read-only scope scoped to relevant services only. Document in deployment runbook. Not a code change.

### PASS — Timeout enforced

`AbortController` is set with `TIMEOUT_MS = 10_000` (10 seconds). The timeout is cleared correctly in both success and failure paths via `clearTimeout(timeout)`. A single retry is attempted on timeout, after which the error surfaces. No unbounded hanging possible.

### PASS — Response size not bounded in memory — MINOR

The response from Citadel is loaded entirely into memory via `response.json()`. With `limit` capped at 500 entries, and typical log entries being small JSON objects, this is unlikely to cause problems in practice. However, if Citadel returns very large `metadata` blobs per entry or a large `episodes` array, memory usage could spike.

**Action:** This is acceptable for V1. A follow-on task could add response body size validation or streaming. Not a blocker.

### PASS — CITADEL_BASE_URL trailing slash stripped

The code does `(getEnv('CITADEL_BASE_URL') ?? DEFAULT_BASE_URL).replace(/\/$/, '')` before building the URL. This prevents double-slash URLs but does not validate the scheme or host.

**SSRF risk:** If an operator sets `CITADEL_BASE_URL` to an internal network address (e.g. `http://169.254.169.254`), the tool would make requests there. This is an operator-level concern, not exploitable by Discord users (who control query/time_range/limit, not the base URL).

**Action:** Document in deployment config that `CITADEL_BASE_URL` must point to the authorized Citadel instance only. Consider adding a URL scheme allowlist (`https://` only) in a future hardening pass. Not a code blocker for V1.

### PASS — Error messages do not leak internal paths or stack traces

All `catch` blocks surface `err.message` or a controlled string — no `err.stack`, no file paths, no internal state. The `@ts-ignore` comment on `query_error` does not affect runtime behavior.

### PASS — No local file system access

The tool does not read or write any local files. It only reads environment variables and makes outbound HTTP requests. No file injection surface.

### PASS — No eval or dynamic code execution

No `eval()`, `new Function()`, `child_process`, or dynamic import with user input. The CPL query string is never executed locally.

### PASS — Rate limit response handled without blind retry loop

The 429 handler does not implement a blind sleep-retry loop. It surfaces the `retry_after` value and returns an error, leaving retry decisions to the calling agent. This prevents an infinite retry DoS against Citadel.

---

## Security Checklist

| Check | Status |
|---|---|
| API key not logged or echoed | PASS |
| No credential hardcoding | PASS |
| CPL injection contained to API key scope | PASS (by design) |
| Network timeout enforced | PASS |
| SSRF surface documented | PASS (operator concern) |
| No local file access | PASS |
| No eval / dynamic execution | PASS |
| Error messages don't leak internals | PASS |
| No blind retry loop on 429 | PASS |
| Response size bounded (practical) | PASS — minor note |

---

## Verdict

**APPROVED.** No blocking security findings. Two advisory notes documented:
1. Citadel API key scope should be read-only and service-scoped in deployment (operator doc item)
2. Response body size bounding could be added in a future hardening pass

Neither requires a code change before merge.
