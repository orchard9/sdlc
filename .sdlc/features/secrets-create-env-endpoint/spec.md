# Spec: Secrets — POST /api/secrets/envs create-only endpoint

## Problem

The REST API for secrets management has a gap: there is no `POST /api/secrets/envs` endpoint.
Currently, env files can only be created via the CLI (`sdlc secrets env set <name> KEY=VALUE`).
The server API exposes `GET /api/secrets/envs` (list) and `DELETE /api/secrets/envs/:name` (delete),
but nothing to create a new env from the frontend or agent-accessible REST surface.

This blocks any frontend or agent workflow that needs to initialize a new encrypted env file
without dropping to the shell.

## Solution

Add a `POST /api/secrets/envs` endpoint that creates a new env file with an initial set of
`KEY=VALUE` pairs, encrypted to the current configured recipients (`keys.yaml`).

The endpoint must:
1. Accept `{ "env": "<name>", "pairs": { "KEY": "VALUE", ... } }` in the request body.
2. Reject the request with `409 Conflict` if an env with that name already exists.
3. Use the existing `sdlc_core::secrets::write_env` to encrypt the content using current recipients.
4. Return `{ "status": "created", "env": "<name>", "key_names": [...] }` on success.
5. Return `400 Bad Request` if no keys are configured (age encryption would fail).
6. Return `400 Bad Request` if `pairs` is empty (creating an empty env file is not useful).

## Non-Goals

- Decryption / export via REST — the server never holds private keys.
- Updating an existing env (merging pairs) — that is a separate concern (PATCH).
- Re-keying — remains CLI-only.

## Data Shape

### Request

```json
POST /api/secrets/envs
Content-Type: application/json

{
  "env": "production",
  "pairs": {
    "DATABASE_URL": "postgres://...",
    "API_KEY": "sk-..."
  }
}
```

### Response (201 Created)

```json
{
  "status": "created",
  "env": "production",
  "key_names": ["DATABASE_URL", "API_KEY"]
}
```

### Error responses

| Code | Condition |
|---|---|
| 400 | `pairs` is empty |
| 400 | No keys configured (cannot encrypt) |
| 409 | Env with that name already exists |
| 500 | `age` binary not found or encryption failed |

## Implementation Touch Points

- `crates/sdlc-server/src/routes/secrets.rs` — add `create_env` handler
- `crates/sdlc-server/src/lib.rs` — register `.route("/api/secrets/envs", get(...).post(create_env))`

No changes to `sdlc-core` are needed — `secrets::write_env` already supports create semantics when
the file does not exist.

## Acceptance Criteria

1. `POST /api/secrets/envs` with valid body and keys configured creates the `.age` file and returns 201.
2. `POST /api/secrets/envs` for an env that already exists returns 409.
3. `POST /api/secrets/envs` with empty `pairs` returns 400.
4. `POST /api/secrets/envs` with no keys configured returns 400.
5. Route is registered and visible in the server.
6. Unit tests cover cases 1–4 above (with mocked `age` calls where appropriate).
