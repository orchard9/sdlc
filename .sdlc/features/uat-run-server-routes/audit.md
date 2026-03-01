# Security Audit: Server Routes for UAT Run History and MilestoneUatCompleted SSE Event

## Scope

This audit covers the security properties of the two new HTTP endpoints, the new SSE event variant, and the frontend additions.

## Attack Surface Analysis

### New Endpoints

```
GET /api/milestones/{slug}/uat-runs
GET /api/milestones/{slug}/uat-runs/latest
```

**Slug injection:** The slug is passed directly from the URL path to `sdlc_core::milestone::list_uat_runs(root, slug)` and `latest_uat_run(root, slug)`. The `paths::milestone_uat_runs_dir` function constructs the filesystem path by joining the project root with `.sdlc/milestones/<slug>/uat-runs/`. Path components containing `..`, `/`, or null bytes would allow directory traversal.

**Assessment:** `sdlc_core::paths` uses `root.join(".sdlc").join("milestones").join(milestone_slug).join("uat-runs")`. Rust's `Path::join` does not resolve `..` components at the OS level on its own, but `std::fs::read_dir` will follow them on most OSes. However, this is an existing pattern used by all milestone and feature routes throughout the codebase — there is no slug sanitization in any existing handler. The threat model for this server is local development use only (not internet-exposed without the tunnel auth layer). The tunnel auth middleware gates all unauthenticated requests.

**No new risk introduced** relative to existing routes.

**Read-only:** Both new endpoints are `GET` only. They do not write, modify, or delete any data. The worst-case impact of a successful exploit is reading UAT run metadata from `.sdlc/`.

### SSE Event: MilestoneUatCompleted

The new `MilestoneUatCompleted { slug }` event carries only the milestone slug, which is already known to any authenticated SSE subscriber. No sensitive data (credentials, file contents, tokens) is included in the event payload.

The event is broadcast to all connected SSE clients when a UAT agent run finishes. All SSE subscribers have already authenticated through the same tunnel auth gate as HTTP endpoints. No escalation of privilege.

**No new risk introduced.**

### Frontend Additions

The TypeScript types (`UatVerdict`, `UatRun`) are pure data declarations. The API client methods are thin wrappers around `fetch`. They do not store, cache, or transmit data outside the normal request/response cycle.

The `request<T>` wrapper throws on non-2xx responses, so `getLatestMilestoneUatRun` will throw a `404` error when no runs exist rather than returning `null` silently — this is consistent with other endpoints and does not introduce a silent failure mode.

**No new risk introduced.**

## Data Exposure

UAT run data (stored in `.sdlc/milestones/<slug>/uat-runs/<id>/run.yaml`) contains:
- Run ID, milestone slug, timestamps
- Test counts and verdict
- Path to the playwright report (a filesystem path string)
- Path to the summary markdown
- List of task slugs created during the run

None of this data is a secret. It is already accessible to any user with filesystem access to the `.sdlc/` directory. Exposing it via HTTP to authenticated tunnel clients is appropriate.

## Authentication

Both new routes are subject to the existing `TunnelAuthLayer` middleware, which:
- Allows all local requests (same host, loopback address)
- Requires a valid bearer token or session cookie for remote requests via tunnel

No changes to authentication were made in this feature.

## Summary

| Risk Category | Assessment |
|---|---|
| Path traversal | Same risk as all existing routes; accepted |
| Data disclosure | Only UAT metadata, not secret; local-dev threat model |
| Authentication bypass | None; routes behind existing auth middleware |
| Data modification | Not applicable; read-only endpoints |
| SSE data leakage | Slug only; already known to subscribers |
| Injection (SQL, etc.) | No database; filesystem only |

**Conclusion:** No new security issues introduced. The implementation is consistent with existing security posture. Approved.
