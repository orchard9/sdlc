# Spec: POST /api/hub/create-repo — Gitea repo creation with HTTP push credentials

## Problem

A developer with a local git repo has no self-serve path to add it to the cluster from
the hub UI. The only onboarding path currently is `POST /api/hub/import`, which requires
an existing remote URL (GitHub). Local-only projects are excluded.

## Solution

Add a `POST /api/hub/create-repo` endpoint that creates a new empty repo in the Gitea
`orchard9/` org and returns a push URL with embedded HTTP credentials. The caller can
immediately add this URL as a git remote and push — no separate credential setup required.

## Endpoint

**`POST /api/hub/create-repo`**

### Request
```json
{ "name": "my-project" }
```

- `name` — required, alphanumeric + hyphens, becomes the Gitea repo slug

### Response (200)
```json
{
  "repo_slug": "my-project",
  "push_url": "http://claude-agent:<token>@<gitea-host>/orchard9/my-project.git",
  "gitea_url": "http://<gitea-host>/orchard9/my-project",
  "provision_triggered": true
}
```

### Error cases
- `400` — name missing, empty, or contains invalid characters
- `409` — repo already exists in Gitea (`conflict` error code)
- `503` — not in hub mode, or Gitea not configured

## Credential Strategy

Use the existing admin Gitea token (already available as `app.gitea_token`) embedded
in the push URL as HTTP basic auth:

```
http://claude-agent:<GITEA_TOKEN>@<gitea-host>/orchard9/<name>.git
```

**Rationale:** This is an internal cluster tool for a single operator (Jordan). A shared
admin token per hub is acceptable. There is no external user surface. If multi-user access
is needed later, this can be upgraded to per-repo deploy keys or per-user tokens.

The Gitea admin username is retrieved at response time via `GET /api/v1/user` using the
admin token, so it doesn't need to be hard-coded.

## Provisioning

After repo creation, call `trigger_provision(repo_slug)` if Woodpecker is configured.
This mirrors the behavior of `POST /api/hub/import`. If provisioning fails (Woodpecker
not configured or call fails), log a warning and return success anyway — provisioning
can be triggered manually from the UI.

## Fleet.rs changes

New functions in `crates/sdlc-server/src/fleet.rs`:

1. **`create_gitea_repo(http_client, gitea_url, gitea_token, name) -> Result<GiteaRepo, FleetError>`**
   - `POST /api/v1/orgs/orchard9/repos` with `{ "name": name, "private": false, "auto_init": false }`
   - Returns `GiteaRepo` on success
   - Maps 409 HTTP status from Gitea to new `FleetError::RepoAlreadyExists(String)`

2. **`get_gitea_username(http_client, gitea_url, gitea_token) -> Result<String, FleetError>`**
   - `GET /api/v1/user` using admin token
   - Returns the login name (e.g. `claude-agent`)

## Routes/hub.rs changes

New handler `pub async fn create_repo(...)`:
- Validates `name`: non-empty, matches `[a-z0-9][a-z0-9-]*` (slugify if needed)
- Calls `fleet::create_gitea_repo()`
- Calls `fleet::get_gitea_username()` to build push URL
- Optionally calls `fleet::trigger_provision()` if Woodpecker configured
- Returns JSON response

New route registration in the hub router (alongside `import`).

## Error handling

- `FleetError::RepoAlreadyExists` → `409 Conflict` with `{ "error": "repo_exists", "detail": "..." }`
- Invalid name → `400 Bad Request` with `{ "error": "invalid_request", "detail": "name must match [a-z0-9][a-z0-9-]*" }`
- Not hub mode → `503` (existing `not_hub_mode()` helper)
- Gitea not configured → `503` (existing `gitea_not_configured()` helper)

## Out of scope

- Per-repo SSH deploy keys
- Per-project Gitea user creation
- Automatic git hook setup on the client side
- Webhook from Gitea push to trigger reconcile (future enhancement)
