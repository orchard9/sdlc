# Design: Secrets — POST /api/secrets/envs create-only endpoint

## Overview

This is a targeted addition of one new HTTP handler to the existing secrets routes file. No new
modules, no new data types in `sdlc-core`, and no schema changes — the entire implementation is a
new function in `crates/sdlc-server/src/routes/secrets.rs` plus one route registration in
`crates/sdlc-server/src/lib.rs`.

## Request/Response Contract

### POST /api/secrets/envs

**Request body** (`application/json`):
```json
{
  "env": "production",
  "pairs": {
    "DATABASE_URL": "postgres://host/db",
    "API_KEY": "sk-abc123"
  }
}
```

**Success response** (`201 Created`):
```json
{
  "status": "created",
  "env": "production",
  "key_names": ["DATABASE_URL", "API_KEY"]
}
```

**Error responses**:
| HTTP Status | Condition |
|---|---|
| 400 Bad Request | `pairs` is empty |
| 400 Bad Request | No AGE keys configured — cannot encrypt |
| 409 Conflict | Env with that name already exists |
| 500 Internal Server Error | `age` binary not found or encryption failed |

## Implementation Design

### Request Body Struct

```rust
#[derive(serde::Deserialize)]
pub struct CreateEnvBody {
    pub env: String,
    pub pairs: std::collections::HashMap<String, String>,
}
```

Using `HashMap<String, String>` for pairs keeps JSON parsing simple and idiomatic. Key order in the
encrypted file is determined by HashMap iteration order (non-deterministic), which is fine — the
file content is opaque ciphertext and only the meta sidecar (which is re-derived from the decrypted
key names) is read by the server.

### Handler Logic

```
fn create_env(app, body):
    1. Validate pairs is non-empty → 400
    2. Load keys from keys.yaml
    3. Validate keys is non-empty → 400
    4. Check if .sdlc/secrets/envs/{env}.age already exists → 409
    5. Build KEY=VALUE string from pairs
    6. Call secrets::write_env(root, env_name, content, keys)
    7. Load updated meta to get key_names
    8. Return 201 with { status, env, key_names }
```

Step 4 checks the `.age` file existence before calling `write_env` to provide a clean 409 — this
avoids relying on an implicit error from the core layer. The check is non-atomic, which is acceptable
given that secrets env creation is a human-initiated operation and race conditions are not a concern.

### Route Registration

The `/api/secrets/envs` route currently has only `get`. It will be chained with `post`:

```rust
// Before:
.route("/api/secrets/envs", get(routes::secrets::list_envs))

// After:
.route("/api/secrets/envs", get(routes::secrets::list_envs).post(routes::secrets::create_env))
```

### Error Mapping

`AppError` is returned for all failure cases. The existing `secrets::write_env` returns
`SdlcError::AgeEncryptFailed` if `age` is missing or encryption fails — `AppError` wraps this as a
500. The 400 and 409 cases are handled before calling into `sdlc-core`.

Axum's `AppError` impl uses `anyhow::Error` internally. We use `anyhow::anyhow!` for 400/409 with
appropriate status-code override via the existing `AppError` → `IntoResponse` impl.

Wait — let me check the current `AppError` impl to understand how to produce non-500 responses:

Looking at the error module: `AppError` wraps `anyhow::Error`. The `IntoResponse` for `AppError`
maps known `SdlcError` variants (like `SecretEnvNotFound`) to 404. For 400 and 409, we'll use
`axum::response::Response` directly or introduce `StatusCode` + `Json` tuple responses.

The cleanest pattern consistent with the existing code is to return a `Result<(StatusCode, Json<Value>), AppError>` where the 201 case returns the tuple and errors are `AppError`:

```rust
pub async fn create_env(
    State(app): State<AppState>,
    Json(body): Json<CreateEnvBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if body.pairs.is_empty() {
        return Err(AppError(anyhow::anyhow!("pairs must not be empty")));
        // ^ This returns 500 by default — need to check how to force 400
    }
    ...
}
```

Actually, looking more carefully at `AppError`: it returns 500 for generic `anyhow::Error` and
404/409 only for specific `SdlcError` variants. The right approach for 400 and 409 is to add new
`SdlcError` variants OR to use Axum's `(StatusCode, Json<Value>)` return type directly.

To keep changes minimal: use `(StatusCode, Json<serde_json::Value>)` as the `Err` arm is not
needed since we can return the non-201 cases as early returns with `Ok((StatusCode::BAD_REQUEST, Json(...)))`:

```rust
pub async fn create_env(
    State(app): State<AppState>,
    Json(body): Json<CreateEnvBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Validate pairs non-empty
    if body.pairs.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "pairs must not be empty" })),
        ));
    }
    let root = app.root.clone();
    let env_name = body.env.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::secrets::load_config(&root)?;
        if config.keys.is_empty() {
            return Err(sdlc_core::SdlcError::AgeEncryptFailed(
                "no keys configured".to_string(),
            ));
        }
        // Check existence
        let env_path = sdlc_core::paths::secrets_env_path(&root, &env_name);
        if env_path.exists() {
            return Err(sdlc_core::SdlcError::SecretEnvExists(env_name.clone()));
        }
        // Build KEY=VALUE content
        let content: String = body.pairs.iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        sdlc_core::secrets::write_env(&root, &env_name, &content, &config.keys)?;
        let meta = sdlc_core::secrets::load_env_meta(&root, &env_name)?;
        Ok(meta.key_names)
    }).await.map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "created",
            "env": body.env,
            "key_names": result,
        })),
    ))
}
```

The `SecretEnvExists` variant needs to be added to `SdlcError` with a 409 mapping in `AppError`'s
`IntoResponse` — OR we handle the 409 before calling `spawn_blocking` by checking existence in the
handler before the blocking task.

**Decision:** Check existence in the blocking task (for correct async semantics) but handle
`SecretEnvExists` in `AppError`'s `IntoResponse` impl with 409. This adds one new `SdlcError`
variant and one 409 match arm.

## Files to Change

| File | Change |
|---|---|
| `crates/sdlc-core/src/error.rs` | Add `SecretEnvExists(String)` variant |
| `crates/sdlc-server/src/error.rs` | Add 409 mapping for `SecretEnvExists` in `AppError::into_response` |
| `crates/sdlc-server/src/routes/secrets.rs` | Add `CreateEnvBody` struct + `create_env` handler |
| `crates/sdlc-server/src/lib.rs` | Chain `.post(routes::secrets::create_env)` onto the envs route |

## Tests

Unit tests in `secrets.rs` using `AppState::new(tmp_dir)`:

1. `create_env_with_no_keys_returns_bad_request` — assert 400 when keys.yaml absent
2. `create_env_with_empty_pairs_returns_bad_request` — assert 400
3. `create_env_when_env_exists_returns_conflict` — requires `age` binary; skip if not installed
4. `create_env_success` — requires `age` binary; skip if not installed

Tests 3 and 4 are gated on `which::which("age").is_ok()` to avoid CI failures on machines without
`age` installed.
