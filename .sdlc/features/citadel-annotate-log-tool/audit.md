# Security Audit: citadel_annotate_log Agent Tool

## Threat Surface

This tool introduces a write path from Pantheon agents to Citadel. Annotations are permanent (no delete API), visible to the entire team in Citadel, and carry an `author_type` field that distinguishes human from AI authorship. The primary security concerns are:

1. **author_type forgery** — can an agent fake a human-authored annotation?
2. **Credential exposure** — is the Citadel API key ever logged or returned to the agent?
3. **Content injection** — can annotation content contain payloads that affect Citadel's rendering?
4. **Approval bypass** — can an agent call this tool without triggering the approval gate?
5. **Uncontrolled writes** — can an agent annotate arbitrary log IDs it didn't fetch?
6. **Rate / volume abuse** — can an agent create unlimited annotations?

---

## Finding A1: author_type Integrity — PASS

**Threat:** Agent passes `author_type: "human"` in tool params, causing Citadel to attribute the annotation to a human user.

**Verification:** The config template contains `"author_type": "ai_agent"` as a Go string literal. The template engine does not expose `{{.author_type}}` as a substitution target. Even if an agent passes `author_type` as a parameter, the ToolExecutor's `RenderTemplate` ignores parameters not referenced in the template — they are silently dropped before the HTTP call.

**Test confirmation:** `TestMapResponse_AuthorTypeHardcoded` verifies the rendered body always contains `"author_type":"ai_agent"` regardless of what additional params are passed.

**Verdict:** No forgery possible via the API. This is the critical invariant and it holds.

**Action:** None.

---

## Finding A2: Credential Exposure — PASS

**Threat:** The Citadel API key is logged in plaintext, included in error responses, or returned in the tool result.

**Verification:**
- The executor's `RecordAppAction` explicitly redacts the credential from the logged params.
- Error responses from `MapResponse` contain only human-readable messages, not the raw HTTP request headers (which contain the Bearer token).
- The tool result struct does not include the credential field.
- `{{.credential}}` is rendered into the HTTP `Authorization` header only — it is not rendered into the response body or any logged field.

**Verdict:** Credential is not exposed.

**Action:** None.

---

## Finding A3: Content Injection — ACCEPTED RISK (documented)

**Threat:** Agent-supplied `content` field contains Markdown or HTML that, when rendered in Citadel's dashboard, triggers XSS or displays misleading information.

**Current state:** The `content` field is passed verbatim to Citadel's `POST /api/v1/annotations` body. Citadel's dashboard is responsible for sanitizing annotation content before rendering.

**Assessment:** Pantheon's responsibility ends at the API boundary. Citadel's annotation endpoint should sanitize content on ingest or at render time. If Citadel has an XSS vulnerability in annotation rendering, that is a Citadel security issue — not a Pantheon issue.

**Mitigating factor:** The tool limits `content` to 4096 chars (validated in `ValidateParams`), preventing extremely large payloads. The `author_type: "ai_agent"` label provides explicit attribution so users know the annotation came from an AI system.

**Action:** Document this in the known limitations: Citadel is responsible for content sanitization. Accepted risk — no change required in Pantheon.

---

## Finding A4: Approval Gate Integrity — PASS

**Threat:** A compromised or misconfigured agent bypasses the `requires_approval = true` gate and annotates Citadel logs without human confirmation.

**Verification:** The executor's approval gate is enforced before `ValidateParams`, `RenderTemplate`, and `Dispatch` are called. The gate checks an approval token that must be provided by a separate approval API call — it cannot be self-signed by the agent. This is the existing Pantheon App Platform approval mechanism used by all `requires_approval: true` tools.

**Verdict:** Approval gate is enforced at the executor layer, not the tool layer. A tool cannot opt out of it.

**Action:** None.

---

## Finding A5: Uncontrolled Write Surface — ACCEPTED RISK (by design)

**Threat:** An agent annotates a log entry it did not actually query — e.g., annotates a log from a different service, tenant, or time period by guessing or constructing a `log_id`.

**Current state:** The tool accepts any `log_id` string. Citadel will return 404 if the ID does not exist, or 200 if it does — regardless of whether the calling agent fetched that log entry via `citadel_query_logs`.

**Assessment:** Pantheon cannot enforce "you may only annotate logs you fetched" without a server-side session context linking query results to subsequent annotation calls. This is a stateless API tool. The agent skill instructions say "Never annotate without the log_id from citadel_query_logs results" — but this is instructional, not enforced.

**Mitigating factor:** The approval gate (`requires_approval: true`) ensures a human confirms the `log_id` and annotation intent before the write fires. Citadel's own access control (tenant_id in headers) prevents cross-tenant annotation.

**Action:** Accepted by design. Document in the tool's known limitations: log_id is not validated against a prior query session. The approval gate is the human-in-the-loop control for this.

---

## Finding A6: Rate / Volume Abuse — OPEN (track as task)

**Threat:** An agent in a loop annotates thousands of log entries, polluting Citadel with AI-generated noise.

**Current state:** No rate limit is enforced at the Pantheon tool level. Each annotation call requires approval, which is a human bottleneck — but if the approval mechanism is automated or has a high rate limit, bulk annotation is possible.

**Assessment:** The approval gate is the primary throttle. If Citadel has rate limiting on its annotations endpoint, that provides a secondary limit. This is a real risk in a future where approval is delegated to an automated policy engine.

**Action:** File task: "Add per-tool rate limit config to ToolDefinition (calls per minute) — citadel_annotate_log should default to 10/min to prevent bulk annotation loops." Non-blocking for this release.

---

## Finding A7: Tenant ID Injection — PASS

**Threat:** Agent supplies a crafted `tenant_id` parameter that routes the annotation to a different organization's Citadel workspace.

**Verification:** The `tenant_id` in the config template (`X-Tenant-ID: {{.tenant_id}}`) is drawn from the `ToolDefinition` config, not from agent-supplied params. It is set at registration time as part of the tool's static config — not exposed as a parameter in the tool's parameter schema.

**Verdict:** Agents cannot change the tenant_id. It is config-bound.

**Action:** None.

---

## Summary

| Finding | Status | Action |
|---|---|---|
| A1: author_type forgery | PASS | None |
| A2: Credential exposure | PASS | None |
| A3: Content injection | ACCEPTED RISK | Document — Citadel responsible for sanitization |
| A4: Approval gate bypass | PASS | None |
| A5: Uncontrolled write surface | ACCEPTED RISK | Document — approval gate is the control |
| A6: Rate/volume abuse | OPEN | Task filed — add per-tool rate limit config |
| A7: Tenant ID injection | PASS | None |

**Verdict: APPROVE**

All critical findings pass. Two accepted risks are documented with rationale. One non-blocking task filed for rate limiting. This feature does not introduce new attack vectors beyond what the existing App Platform executor already handles. The author_type integrity guarantee (Finding A1) is the key invariant and it holds.
