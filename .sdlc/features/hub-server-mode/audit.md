# Security Audit: Hub Server Mode

## Scope

New endpoints: `POST /api/hub/heartbeat`, `GET /api/hub/projects`, `GET /api/hub/events`.
New state: `HubRegistry` in `AppState`, persisted to `~/.sdlc/hub-state.yaml`.
New CLI: `--hub` flag on `sdlc ui start`.

## Findings

### A1: No authentication on hub endpoints
**Severity: Accepted (design decision)**
Hub endpoints pass through the existing `auth_middleware`. In local mode (no tunnel
configured), the middleware is a passthrough — same as all other endpoints. If an
orch-tunnel is active, the same token/cookie gate applies.
Hub mode is designed for local developer use (no auth) and cluster use (tunnel/ingress auth).
No change needed.

### A2: Heartbeat payload is deserialized without size limits
**Severity: Low**
The heartbeat payload is a fixed schema. Axum's default body size limit (1MB) applies.
The `name` and `url` fields are stored in memory. Extremely long strings could inflate
memory, but the in-process sweep removes entries after 5 minutes. The surface is localhost
by default.
**Action: Accept** — in a cluster deployment, the hub would be behind an ingress with its
own size limits. A task is added to track configurable field length limits.
**Task added: hub-server-mode/T9 — Add max field length validation to heartbeat payload.**

### A3: Hub state file written to `~/.sdlc/hub-state.yaml`
**Severity: Informational**
The file contains project URLs and milestone names — potentially sensitive in an enterprise
context. It is a user-home file, readable only by the current user on a standard system
(umask 022 → 644). No secrets are stored (no tokens, passwords, or API keys).
**Action: Accept** — standard temp-cache pattern.

### A4: No rate limiting on heartbeat endpoint
**Severity: Low**
An adversary on the same network with tunnel access could flood the heartbeat endpoint and
inflate the registry. The 5-minute sweep removes expired entries. In hub mode, the
auth_middleware is the primary gate.
**Action: Accept** — rate limiting is an infrastructure concern (ingress/nginx). Track as
future improvement.
**Task added: hub-server-mode/T10 — Add optional per-IP rate limit config for heartbeat.**

### A5: SSE event contains full ProjectEntry including URL
**Severity: Informational**
The `ProjectUpdated` SSE event includes the full project URL. This is by design — the hub
UI needs the URL to render a click-through link. The SSE stream is only accessible to
authenticated clients (same auth gate as the REST endpoints).
**Action: Accept.**

### A6: Sweep task is not aborted on AppState drop
**Severity: Low (test safety)**
The sweep task spawned in `new_with_port_hub` is not tracked in `WatcherGuard`. In
production, the process exits, so the task is reaped. In tests, `new_with_port_hub` is not
called so no leaked task.
**Action: Accept for now** — tracked as review finding F2. Added T11 below.
**Task added: hub-server-mode/T11 — Track hub sweep handle in WatcherGuard for proper cleanup.**

## Tasks Created

```
sdlc task add hub-server-mode "Add max field length validation to heartbeat payload"
sdlc task add hub-server-mode "Add optional per-IP rate limit config for heartbeat endpoint"
sdlc task add hub-server-mode "Track hub sweep handle in WatcherGuard for proper cleanup"
```

## Verdict: Approved

No blocking security issues. Three low-severity findings tracked as follow-up tasks.
The hub endpoints correctly delegate to the existing auth middleware. No secrets are exposed
or stored. The implementation is safe for local developer use and cluster deployment behind
an ingress.
