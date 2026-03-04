# Security Audit: dev-driver-agentic-dispatch

## Scope

This audit covers the security surface introduced by this feature:
1. New `POST /api/tools/agent-dispatch` Rust endpoint
2. New `runAgentDispatch()` TypeScript helper in `_shared/agent.ts`
3. Changes to `dev-driver/tool.ts` dispatch logic

---

## Threat Model

### A. Unauthorized agent dispatch via `POST /api/tools/agent-dispatch`

**Risk:** An attacker calls the endpoint to spawn arbitrary agent runs.

**Mitigations:**
- Bearer token validation is the first gate — matches the same `app.agent_token` pattern used by `agent-call`
- `app.agent_token` is generated at server startup via `generate_agent_token()` using `/dev/urandom` (16 bytes, hex-encoded) — 128 bits of entropy
- The token is not logged or emitted in SSE events
- Token is only injected into tool subprocess environments; it is not accessible from the frontend API

**Finding:** No issue. Protection is equivalent to `agent-call`.

---

### B. Prompt injection via `prompt` field

**Risk:** A malicious `prompt` value could cause the agent to perform unintended actions.

**Mitigations:**
- Only authenticated callers (valid bearer token) can reach this endpoint
- The `prompt` is passed directly to `spawn_agent_run` — the agent has the same tool access (`Bash`, `Read`, `Write`, etc.) as any other server-spawned run
- Dev-driver only calls this endpoint internally with hardcoded command templates (`/sdlc-next <slug>` and `/sdlc-run-wave <milestone>`), where slug/milestone values come from `sdlc next --json` and `sdlc milestone list --json` — CLI commands that return only valid slugs

**Finding:** The endpoint itself has no prompt sanitization, but that is consistent with all other agent endpoints (ponder, investigation, feature runs). The attack surface requires a valid bearer token, which is a sufficient gate for an internal-only tool endpoint.

**Finding:** `run_key` and `label` values are not validated against an allow-list. A caller with a valid token could supply arbitrary values. This does not change the security posture — the slug/milestone constraint is enforced by the calling TypeScript code, and the bearer token is the principal boundary.

---

### C. Resource exhaustion via unlimited agent spawns

**Risk:** A caller with a valid token spawns many concurrent agent runs.

**Mitigations:**
- The `run_key` deduplication check in `spawn_agent_run` limits concurrent runs per key to one
- The `agent_runs` map is the only concurrency limiter — there is no global max-concurrent-runs limit
- This is the same situation as all other agent endpoints; no regression introduced here

**Finding:** No new risk relative to baseline. Accepted.

---

### D. `SDLC_AGENT_TOKEN` exposure in tool subprocess environment

**Risk:** The token is injected into every tool subprocess environment, making it accessible to tool code.

**Mitigations:**
- This is by design — tools need the token to call `/api/tools/agent-dispatch` and `/api/tools/agent-call`
- Tool code runs on the same host as the server; the security boundary is the server's token validation, not process isolation
- `runAgentDispatch()` reads `SDLC_AGENT_TOKEN` from `process.env` — it does not store it, log it, or transmit it to any other endpoint

**Finding:** No new risk. The environment injection pattern is existing behavior.

---

### E. Lock file removal — no security regression

The old `.sdlc/.dev-driver.lock` file was a world-writable JSON file in the project root. Removing it eliminates a local file that any process on the machine could overwrite to spoof flight state. The new mechanism (server-side `agent_runs` map) is in-memory and only modifiable via authenticated HTTP.

**Finding:** Security posture improved. No regression.

---

## Summary

| Finding | Status | Action |
|---------|--------|--------|
| Unauthorized dispatch: protected by bearer token | No issue | None |
| Prompt injection: token is sufficient gate | Accepted | None |
| Unlimited concurrent runs: same as baseline | Accepted | None |
| Token in subprocess env: by design | No issue | None |
| Lock file removal: improves posture | Improvement | None |

All findings are accepted. No blocking security issues.
