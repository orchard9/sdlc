# Tasks: hub-create-repo-api

## Task List

1. Add `FleetError::RepoAlreadyExists` variant to `fleet.rs`
2. Add `create_gitea_repo()` function to `fleet.rs`
3. Add `get_gitea_username()` function to `fleet.rs`
4. Add `CreateRepoRequest` / `CreateRepoResponse` types and `create_repo` handler to `routes/hub.rs`
5. Register `POST /api/hub/create-repo` route in the router
6. Add unit tests for `create_gitea_repo` and `get_gitea_username` in `fleet.rs`
