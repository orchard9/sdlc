# Audit: run-activity-ui

## Security surface

### Backend changes

**`RunRecord.prompt` field**: Stores the first 2000 characters of the agent prompt. The prompt originates from internal server-side logic (the prompt passed to `spawn_agent_run`), not from user-controlled input in HTTP requests. The field is truncated to 2000 chars to avoid storing excessively large data on disk.

**`GET /api/runs/:id/telemetry`**: Read-only endpoint. Returns data already stored in the existing events sidecar (`{id}.events.json`) plus the prompt from the RunRecord. No new write surface. Authentication is handled by the existing tunnel auth middleware (applied to all `/api/*` routes). The endpoint is subject to the same authorization as `GET /api/runs/:id`.

**No XSS risk**: Prompt text is rendered as React text content (not dangerouslySetInnerHTML), so it is automatically escaped.

**No injection risk**: The telemetry endpoint reads from the existing sidecar files by run ID. The run ID is validated to be a timestamp-based ID before any file I/O (existing validation in `load_run_events`).

### Frontend changes

All data is displayed as text via React's standard rendering — no HTML injection possible. JSON input blobs for tool calls are rendered via `JSON.stringify` inside a `<pre>` element, not via innerHTML.

## Findings

None. This feature is a read-only UI addition over existing persisted data. The security surface is minimal and consistent with existing endpoints.

## Verdict

Approved.
