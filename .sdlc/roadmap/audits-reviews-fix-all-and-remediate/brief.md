We need tighter control about what we issue fix all and remediate to, for example, this is after an audit. everything in this list we should either remediate or fix. this is a template chance, a concept change that deserves to be in claude.md, and a guidance.md adjustment

be token efficient in your changes to instruction files like guidance.md and templates

Security Audit: WebhookRoute Registration and Tick Dispatch
Attack Surface
This feature adds three new HTTP endpoints and extends an existing ingestion endpoint:

POST /api/orchestrator/webhooks/routes — write route to DB
GET /api/orchestrator/webhooks/routes — read routes from DB
POST /webhooks/{route} (existing) — store raw payload bytes in DB
And it extends the sdlc orchestrate daemon's tick loop to read payloads and dispatch tools.

Findings
MEDIUM — No authentication on /webhooks/{route} ingestion endpoint
What: POST /webhooks/{route} accepts arbitrary payloads from any caller — no HMAC signature verification, no IP allowlist, no token. Any party who knows the URL can inject payloads into the DB, which the tick loop will consume on the next tick.

Impact: An attacker who can reach the sdlc-server (typically localhost:3141 in development, or the public cloudflared tunnel URL if tunnel is enabled) can inject arbitrary webhook payloads. The tick loop will then call a registered tool with attacker-controlled input. If a route is registered for the path, this is a SSRF / arbitrary tool invocation vector.

Mitigation (not implemented — future work): HMAC signature verification (e.g. X-Hub-Signature-256 as used by GitHub) per route. The route input_template could also carry an expected content-type or a secret field for validation.

Current residual risk: Low in practice because:

The sdlc-server is a local development tool, bound to 0.0.0.0 but typically only exposed via the SDLC tunnel (Cloudflare Quick Tunnels), which requires a token for the management API.
The ingestion endpoint predates this feature and already stored arbitrary payloads.
Tool invocation is limited to .sdlc/tools/<name>/tool.ts — attacker cannot invoke arbitrary binaries; the tool_name is constrained by registered routes (not by the attacker).
Tracking: Add as a task for the authentication follow-on feature.

LOW — No validation on tool_name in route registration
What: RegisterRouteBody.tool_name is stored as-is without checking that the tool exists or that the name is a valid tool slug (no path traversal check).

Impact: A route could be registered with tool_name = "../evil" or tool_name = "nonexistent". At dispatch time, tool_script(root, &route.tool_name) resolves to .sdlc/tools/../evil/tool.ts — i.e. .. in the name could navigate outside the tools directory.

Investigation: sdlc_core::paths::tool_script is implemented as:

pub fn tool_script(root: &Path, name: &str) -> PathBuf {
    tool_dir(root, name).join("tool.ts")
}
// = root.join(".sdlc/tools").join(name).join("tool.ts")
On macOS/Linux, root.join(".sdlc/tools").join("../evil") resolves to root.join(".sdlc/evil"). If tool.ts doesn't exist there, dispatch_webhook logs and deletes the payload — no tool is run. The script.exists() check prevents execution.

However, if .sdlc/evil/tool.ts exists (possible on a multi-project filesystem), it would be invoked.

Mitigation (recommended): Add tool_name slug validation in register_route using the existing validate_slug function from sdlc_core::paths. This rejects names with / or ... Implementation is a one-line fix.

Immediate fix added: The register_route handler should call sdlc_core::paths::validate_slug(&body.tool_name) and return 400 on failure. This is a low-risk finding but worth adding.

LOW — Template injection via input_template
What: input_template is stored verbatim and rendered at dispatch time by string-replacing {{payload}} with the JSON-escaped payload. If the operator registers a malicious template that produces a tool input with unexpected structure, the tool may behave unexpectedly.

Impact: Self-inflicted only — the input_template is set by the operator registering the route, who has write access to the sdlc-server API. No external party controls the template content. The rendered input is JSON-parsed before invocation, so non-JSON templates fail safely (payload deleted, dispatch skipped).

Verdict: No action required. The operator who registers a route is trusted.

LOW — Payload bytes stored indefinitely if daemon not running
What: Webhook payloads are stored in WEBHOOKS until the tick loop runs. If the sdlc orchestrate daemon is not running, payloads accumulate in the DB indefinitely.

Impact: The DB could grow without bound on a long-lived sdlc-server instance that never runs the daemon. No data leakage — payloads are local files.

Mitigation: Future work — add a max-age TTL for stored payloads. No action in this feature.

Required Fix Before Ship
Add tool_name slug validation in register_route. This prevents path traversal in the tool lookup. The fix is:

// In register_route, after existing validation:
if let Err(e) = sdlc_core::paths::validate_slug(&body.tool_name) {
    return Err(AppError::bad_request(format!("tool_name: {e}")));
}
This should be implemented before the QA phase.

Unchanged Attack Surface
The management endpoints (POST /api/orchestrator/webhooks/routes, GET /api/orchestrator/webhooks/routes) are behind the existing SDLC server auth middleware (token/cookie gate, local bypass) — the same protection as all other /api/* routes. No new auth bypass is introduced.

Verdict
APPROVE with one required fix: add validate_slug check for tool_name in register_route. The HMAC authentication finding is tracked for follow-on work and does not block ship for a local development tool.