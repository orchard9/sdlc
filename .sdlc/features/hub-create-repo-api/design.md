# Design: POST /api/hub/create-repo

Backend-only feature. No UI components — the frontend work is in `hub-create-repo-ui`.

## Data Flow

```
POST /api/hub/create-repo
  { "name": "my-project" }
       │
       ▼
routes/hub.rs :: create_repo()
  1. Validate name (non-empty, slug chars)
  2. fleet::create_gitea_repo(name)         → GiteaRepo
  3. fleet::get_gitea_username()             → "claude-agent"
  4. Build push_url: http://claude-agent:<token>@<host>/orchard9/<name>.git
  5. fleet::trigger_provision(name)          → fire-and-forget (warn on fail)
  6. Return JSON response
       │
       ▼
200 { repo_slug, push_url, gitea_url, provision_triggered }
```

## New fleet.rs Functions

### `create_gitea_repo`

```
POST {gitea_url}/api/v1/orgs/orchard9/repos
Authorization: token {gitea_token}
{
  "name": name,
  "private": false,
  "auto_init": false,
  "description": ""
}
```

- 201 → deserialize as `GiteaApiRepo`, return `GiteaRepo`
- 409 → return `FleetError::RepoAlreadyExists(name)`
- other → return `FleetError::GiteaUnavailable(format!("HTTP {status}: {detail}"))`

### `get_gitea_username`

```
GET {gitea_url}/api/v1/user
Authorization: token {gitea_token}
```

Returns `login` field from response. Cached for the lifetime of a request (no persistent cache needed — called once per create-repo invocation).

## New FleetError Variant

```rust
#[error("repo already exists: {0}")]
RepoAlreadyExists(String),
```

- `status_code()` → `409 Conflict`
- `error_code()` → `"repo_exists"`

## Route Handler Skeleton

```rust
pub async fn create_repo(
    State(app): State<AppState>,
    Json(req): Json<CreateRepoRequest>,
) -> axum::response::Response {
    // 1. Hub mode check
    // 2. Gitea config check
    // 3. Validate name (regex: ^[a-z0-9][a-z0-9-]*$, max 100 chars)
    // 4. fleet::create_gitea_repo(...)
    // 5. fleet::get_gitea_username(...)
    // 6. Build push_url
    // 7. fleet::trigger_provision(...) — warn on failure, don't fail request
    // 8. Return JSON
}
```

## Router Registration

In `crates/sdlc-server/src/main.rs` (or wherever hub routes are registered), add:

```rust
.route("/api/hub/create-repo", post(hub::create_repo))
```

## Name Validation

Regex: `^[a-z0-9][a-z0-9-]{0,98}[a-z0-9]$` or `^[a-z0-9]$` (single char).
Combined: at least 1 char, starts/ends with alphanumeric, hyphens allowed in middle.
Max length: 100 chars (Gitea limit).

Invalid name → `400 { "error": "invalid_request", "detail": "name must be lowercase alphanumeric with hyphens, max 100 chars" }`.

## Test Coverage

- `create_gitea_repo_success` — mock 201, assert GiteaRepo returned
- `create_gitea_repo_conflict` — mock 409, assert RepoAlreadyExists
- `get_gitea_username_success` — mock 200 with login field
- Route handler validation tests (invalid name, missing name)
