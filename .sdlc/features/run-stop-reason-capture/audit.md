# Security Audit: Capture session_id and stop_reason in RunRecord and telemetry events

## Scope

Additive data-capture change that adds two fields — `session_id` (Claude conversation session identifier) and `stop_reason` (agent termination reason) — to `RunRecord`. Fields are captured from `Message::Result`, persisted to `.sdlc/.runs/*.json`, and emitted in the `RunFinished` SSE event. Frontend TypeScript interface extended with matching optional fields.

---

## Security Surface Analysis

### Data sensitivity: `session_id`

`session_id` is a Claude-generated conversation identifier (opaque string). It allows resuming a prior conversation via `--resume`.

**Risk assessment:** The session ID is already present in the raw event stream persisted to `.sdlc/.runs/*.events.json` — this change adds it to the smaller summary record, not to a new location. If an attacker has read access to `.sdlc/.runs/`, they already have the session ID from the events sidecar. No new attack surface is created by also storing it in the summary record.

The session ID is surfaced to the authenticated frontend via `GET /api/runs`. The server's tunnel auth middleware (`auth.rs`) gates all API access behind a bearer token on remote connections, with a local bypass for loopback requests. No change to the authentication perimeter.

**Finding:** ACCEPTED — no new exposure.

### Data sensitivity: `stop_reason`

`stop_reason` is an operational metadata string (e.g. `"end_turn"`, `"max_turns"`, `"error_max_turns"`). It reveals nothing sensitive about the agent's conversation content, the user's prompts, or the server's internal state beyond what is already visible in the run status field.

**Finding:** ACCEPTED — non-sensitive operational metadata.

### SSE event extension: `RunFinished`

The `RunFinished` SSE variant now includes `session_id` and `stop_reason`. SSE is streamed to authenticated clients only (same auth gate as the REST API). The session ID in the SSE payload is consistent with what is already available via `GET /api/runs`.

**Finding:** ACCEPTED — no new exposure beyond existing authenticated channel.

### Persistence: `.sdlc/.runs/*.json`

Persisted files are local filesystem artifacts. Access control is filesystem-level (same as all other `.sdlc/` content). No change to the file path, permissions model, or retention policy.

**Finding:** ACCEPTED — no change to persistence security posture.

### Backward compatibility: serde defaults

`#[serde(default, skip_serializing_if = "Option::is_none")]` prevents injection of null/empty values into existing records on deserialization. No deserialization of attacker-controlled data is involved in this code path.

**Finding:** ACCEPTED.

### Resume workflow: `session_id` used for `--resume`

The existing `POST /api/runs/:id/resume` endpoint reads `session_id` from the stored `RunRecord` to resume a paused run. This endpoint was already present before this feature and gated by the same auth middleware. This feature ensures `session_id` is reliably populated in all completed/paused `RunRecord`s, which is a correctness improvement for that endpoint — not a new security exposure.

**Finding:** ACCEPTED — improves reliability of an existing authenticated endpoint.

---

## Verdict

**No security findings.** This change captures non-sensitive operational metadata that was already present in the raw event stream. It does not expand the authentication perimeter, introduce new data flows, or expose sensitive content. The change is safe to merge.
