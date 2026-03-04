# Security Audit: hub-heartbeat-client

## Scope

`crates/sdlc-server/src/heartbeat.rs` — background heartbeat task that sends
telemetry to a hub server.

---

## Findings

### F1 — Outbound URL is operator-controlled (SDLC_HUB_URL)
**Severity:** Informational
**Status:** Accepted — by design

`SDLC_HUB_URL` is set by the server operator (the person running `sdlc serve`),
not by end users or HTTP request data. An operator who controls env vars already
has full access to the host. No SSRF vector from untrusted input.

**Action:** None required.

---

### F2 — Payload contains project metadata (name, milestone, feature count)
**Severity:** Informational
**Status:** Accepted — by design

The heartbeat sends the project folder name, active milestone slug, feature count,
and whether an agent is running. This is intentional: the hub server aggregates
this data to provide a project navigator dashboard. The data is sent only when
`SDLC_HUB_URL` is explicitly configured by the operator.

**Action:** None required.

---

### F3 — No authentication on outbound heartbeat POST
**Severity:** Low
**Status:** Accepted — intentional design choice

The heartbeat POST does not include an auth token. The hub server endpoint
(`POST /api/hub/heartbeat`) is public by design — any project instance that
knows the hub URL can register. The data sent is low-sensitivity metadata, not
secrets. If the hub needs auth in the future, `SDLC_HUB_TOKEN` can be added
as an env var and sent as a `Bearer` header.

**Action:** Tracked as a future enhancement. Not a blocking finding.

---

### F4 — 5-second timeout prevents connection exhaustion
**Severity:** N/A (positive finding)

Each heartbeat POST uses a hard 5-second `reqwest` timeout. This ensures the
background task cannot stall the Tokio thread pool if the hub is slow or
unreachable.

---

### F5 — No secrets are transmitted
**Severity:** N/A (positive finding)

The payload contains only: project name (directory basename), public URL
(`SDLC_BASE_URL`), milestone slug, feature count, and agent-running boolean.
No auth tokens, no secrets, no user data, no file contents.

---

### F6 — Task is isolated from the request path
**Severity:** N/A (positive finding)

The heartbeat task runs as an independent `tokio::spawn` loop. Heartbeat
failures (network errors, timeouts, hub downtime) are logged as `warn!` and do
not affect the server's request handling, auth, or state.

---

## Summary

No blocking findings. All data transmitted is operator-configured metadata.
The implementation is well-isolated and uses appropriate timeouts. One
low-severity finding (no auth on outbound POST) is accepted by design.

## Verdict: Approved
