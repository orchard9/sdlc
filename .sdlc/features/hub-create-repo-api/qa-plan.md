# QA Plan: hub-create-repo-api

## Unit Tests (fleet.rs)

| Test | Scenario | Expected |
|---|---|---|
| `create_gitea_repo_success` | Mock Gitea returns 201 with repo JSON | Returns `GiteaRepo` with correct slug/urls |
| `create_gitea_repo_conflict` | Mock Gitea returns 409 | Returns `FleetError::RepoAlreadyExists` |
| `create_gitea_repo_gitea_error` | Mock Gitea returns 500 | Returns `FleetError::GiteaUnavailable` |
| `get_gitea_username_success` | Mock `GET /api/v1/user` returns `{"login":"claude-agent"}` | Returns `"claude-agent"` |
| `get_gitea_username_error` | Mock returns 401 | Returns `FleetError::GiteaUnavailable` |

## Integration Tests (routes/hub.rs)

| Test | Scenario | Expected |
|---|---|---|
| `create_repo_not_hub_mode` | No hub registry | `503` |
| `create_repo_gitea_not_configured` | No gitea config | `503` |
| `create_repo_invalid_name_empty` | `{ "name": "" }` | `400 invalid_request` |
| `create_repo_invalid_name_uppercase` | `{ "name": "MyProject" }` | `400 invalid_request` |
| `create_repo_invalid_name_spaces` | `{ "name": "my project" }` | `400 invalid_request` |
| `create_repo_conflict` | Gitea returns 409 | `409 repo_exists` |

## Compile Check

`SDLC_NO_NPM=1 cargo build --all` passes with no warnings.
`cargo clippy --all -- -D warnings` passes.

## Manual Smoke Test (cluster)

```bash
curl -X POST https://sdlc.threesix.ai/api/hub/create-repo \
  -H "Content-Type: application/json" \
  -d '{"name":"test-onboard-xyz"}' \
  -b "<session-cookie>"
# → 200 with push_url containing claude-agent token
# → Gitea: http://100.79.2.8:30300/orchard9/test-onboard-xyz exists
# → Woodpecker: provision pipeline triggered
```
