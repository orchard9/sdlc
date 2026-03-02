# Security Audit: HTTP Webhook Receiver and Raw Payload Storage in redb

## Surface Area

This feature introduces one new HTTP endpoint and one new redb table. Both are local to the sdlc-server process, which runs on localhost by default (only exposed externally via the optional cloudflared tunnel).

---

## Findings

### AUTH-1: No webhook-sender authentication (accepted risk)

**Severity:** Low (local context)

The `POST /webhooks/{route}` endpoint sits behind the existing sdlc-server auth middleware (`auth_middleware` in `auth.rs`). When a tunnel is active, the tunnel token gates all requests. When running locally, no auth is applied — matching the behavior of all other `/api/*` routes.

There is no HMAC signature verification specific to webhook senders. This is explicitly out of scope per the spec ("Authentication / HMAC signature verification — future feature").

**Accepted:** The current auth model is consistent with the rest of the server. Webhook-specific signature verification is a future hardening step.

---

### STORE-1: Raw body stored verbatim — no size cap (accepted risk)

**Severity:** Low (local process, localhost-only by default)

The handler accepts any body size and stores it to redb without a size limit. A malicious or misconfigured sender could write arbitrarily large payloads to disk.

**Accepted:** The server is a local developer tool, not a public service. Payload size limits are explicitly out of scope in the spec ("No payload size limits — future hardening"). The disk impact is bounded by the user's local disk, consistent with other redb usage in the project.

---

### STORE-2: No payload sanitization or validation (by design)

The spec states "no transformation on ingress — store exactly what arrived." The raw bytes are stored as a JSON byte array in redb. There is no deserialization or execution of the payload content at storage time — it is opaque bytes until the tick loop reads it.

**No risk:** The payload is never executed, parsed as code, or reflected into a browser context at this layer.

---

### STORE-3: redb file permissions

The `orchestrator.db` file is created at `.sdlc/orchestrator.db` with the operating system's default umask. On macOS and Linux this is typically `0644` (world-readable). This is consistent with all other `.sdlc/` files (YAML state files, artifact Markdown) which carry the same permissions.

**No new risk:** No regression from existing behavior.

---

### ROUTE-1: Route path segment not validated

The `{route}` path segment is captured as a `String` and stored in `route_path` without slug validation. A sender could provide `route_path = "../escape"` or other unusual strings.

**No risk at this layer:** The `route_path` is stored as a JSON string value inside the redb value bytes. It is never used as a filesystem path, shell argument, or SQL query at this layer. Future consumers of `route_path` (tick loop dispatch logic) must validate it before using it to select tools.

**Recommendation (future):** Add slug validation to `route_path` when the tick loop routing logic is implemented.

---

## No Critical or High Severity Findings

All findings are low severity and accepted given the local developer tool context and explicit out-of-scope declarations in the spec.

**Recommendation: Approve.**
