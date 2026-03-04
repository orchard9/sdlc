# Security Audit: tool-agent-invocation

## Scope

This audit covers the three components shipped by this feature:

1. `POST /api/tools/agent-call` — new server endpoint for tool-spawned agent calls
2. `SDLC_AGENT_TOKEN` + `SDLC_SERVER_URL` injection into tool subprocess environments
3. `_shared/agent.ts` — the client-side TypeScript library tools use to invoke agents

---

## Threat Model

**Who can reach `/api/tools/agent-call`?**

- Tool subprocesses running on the same host as the server (the intended caller)
- Any process on localhost that knows the port

**Who cannot reach it?**

- Remote callers via the tunnel: the tunnel auth middleware blocks all `/api/*` routes lacking a session cookie or tunnel token. This endpoint adds its own additional guard (SDLC_AGENT_TOKEN bearer), so even if tunnel auth were bypassed, the caller would still fail.

**What can an authorized caller do?**

- Invoke a Claude agent run with an arbitrary `prompt` and optional `agentFile`
- The agent runs with the standard `sdlc_query_options` tool set (no extra MCP servers, no elevated permissions)
- The run appears in the activity feed and is logged in run history

---

## Finding A1 — Token Generation: Adequate Entropy

**Status:** PASS

`generate_agent_token()` (state.rs:373) uses `std::fs::File::open("/dev/urandom")` + `read_exact(&mut [u8; 16])`, producing a 32-character lowercase hex string (128 bits of OS CSPRNG entropy). The comment explicitly warns against using `std::fs::read()` (which would loop forever on `/dev/urandom`). A Windows/CI fallback (nanos + pid) is noted; this is weaker but acceptable for local-only endpoints.

The token is never persisted to disk and changes on every server restart.

**No action required.**

---

## Finding A2 — Token Validation: Correct Pattern

**Status:** PASS

`agent_call()` in tools.rs validates the bearer token via an inline constant-time comparison (`provided != *app.agent_token`). This is a simple string comparison — not constant-time in the cryptographic sense — but is acceptable here because:

- The endpoint is local-only (no network attacker can mount a timing attack from across the internet)
- The token is 32 hex chars (128-bit CSPRNG); brute-force is infeasible
- This matches the pattern already used for tunnel auth

Missing token → 401. Wrong token → 401. Error message does not reveal the expected token.

**No action required.**

---

## Finding A3 — `agentFile` Path Traversal

**Status:** ACCEPTABLE — Accepted by design

`agent_call()` accepts an optional `agentFile` string, joins it to `app.root` with `app.root.join(path)`, and reads the file contents. A path like `../../../../etc/passwd` could read arbitrary files on the filesystem.

**Mitigating factors:**
- The caller must hold a valid `SDLC_AGENT_TOKEN`, which is only injected into tool subprocesses spawned by the server. Tool authors are trusted developers in the same project — not adversarial external callers.
- The file content is prepended to the agent's system prompt (not executed or returned directly to the HTTP client).
- The tunnel auth middleware already prevents external callers from reaching this endpoint.

**Decision:** Accept. The agent token is a legitimate boundary; attackers that can compromise tool subprocesses already have local code execution. If this changes to an externally-accessible surface, add path canonicalization.

**Tracked as future hardening:** `sdlc task add tool-agent-invocation "Harden agentFile path: canonicalize against app.root and reject paths that escape the project tree"` — but this is not a blocker for release given the current trust model.

---

## Finding A4 — `SDLC_AGENT_TOKEN` Exposure via Process Environment

**Status:** PASS — By Design

The token is injected into every tool subprocess's environment (`SDLC_AGENT_TOKEN`). Any tool process can read it. This is intentional — all tools are trusted developer code. The env var is not logged by the server, not written to disk, and not returned to browser clients.

**No action required.**

---

## Finding A5 — Deadlock Risk for Synchronous Tools

**Status:** DOCUMENTED, enforced by convention

The spec explicitly notes that `agent_call` is only safe from tools with `streaming: true`. A synchronous (blocking) tool invoking `runAgent()` would exhaust the blocking thread pool (`spawn_blocking`) by holding a thread while waiting for the agent HTTP response, which itself blocks on a thread completing the agent run. This is a classic deadlock.

The endpoint's doc comment warns against this. The `sdlc-tool-build` skill instructions document the `streaming: true` requirement. The tool schema (`streaming` flag in meta) is the enforcement mechanism.

There is no runtime guard that prevents a synchronous tool from calling the endpoint. If this becomes an operational concern, the endpoint could validate that the caller's interaction record has `streaming_log: true` — but this would require correlating the request with the active interaction, which is not straightforward.

**Decision:** Accept current state. Convention + documentation is sufficient for developer tools. Track as improvement: `sdlc task add tool-agent-invocation "Consider runtime guard: reject /api/tools/agent-call from non-streaming tool contexts"` — post-release.

---

## Finding A6 — `runAgentCli` Uses `--system-prompt` flag

**Status:** INFORMATIONAL

`runAgentCli()` in `_shared/agent.ts` passes the agent's persona as `--system-prompt` to the `claude` CLI. This is the synchronous fallback for non-server contexts. The system prompt content is the agent's `.md` file body (YAML frontmatter stripped). No shell injection risk — `spawnSync` is used with an `args` array, not a shell string.

**No action required.**

---

## Summary

| Finding | Severity | Status |
|---|---|---|
| A1 — Token entropy | - | PASS |
| A2 — Token validation pattern | - | PASS |
| A3 — agentFile path traversal | Low | ACCEPTED (trust model) |
| A4 — Token in subprocess env | - | PASS (by design) |
| A5 — Deadlock for sync tools | Medium | DOCUMENTED (convention enforced) |
| A6 — runAgentCli shell injection | - | PASS |

No blocking findings. Two low-priority hardening items tracked. Feature is safe to release within the current local-developer trust model.

## Verdict

APPROVE
