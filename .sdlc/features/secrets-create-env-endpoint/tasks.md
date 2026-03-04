# Tasks: Secrets — POST /api/secrets/envs create-only endpoint

## T1 — Add `SecretEnvExists` error variant to `sdlc-core`

**File:** `crates/sdlc-core/src/error.rs`

Add `SecretEnvExists(String)` to the `SdlcError` enum, with the display message
`"env already exists: {0}"`. This is the domain error returned when a create request targets an
env file that already exists.

---

## T2 — Implement `create_env` handler in `secrets.rs`

**File:** `crates/sdlc-server/src/routes/secrets.rs`

Add:
- `CreateEnvBody` struct with fields `env: String` and `pairs: HashMap<String, String>`.
- `create_env` async handler that:
  1. Returns 400 if `pairs` is empty.
  2. Loads keys; returns 400 if no keys configured.
  3. Checks env existence; returns 409 (`SecretEnvExists`) if the `.age` file already exists.
  4. Builds `KEY=VALUE\n...` content from `pairs`.
  5. Calls `sdlc_core::secrets::write_env`.
  6. Loads the meta sidecar for key_names.
  7. Returns `(StatusCode::CREATED, Json({ status, env, key_names }))`.

The handler return type is `Result<(StatusCode, Json<serde_json::Value>), AppError>`.

---

## T3 — Map `SecretEnvExists` to 409 in `AppError::into_response`

**File:** `crates/sdlc-server/src/error.rs`

Add `SdlcError::SecretEnvExists(_)` to the 409 match arm in `AppError`'s `IntoResponse` impl.

---

## T4 — Register the new route in `lib.rs`

**File:** `crates/sdlc-server/src/lib.rs`

Change:
```rust
.route("/api/secrets/envs", get(routes::secrets::list_envs))
```
to:
```rust
.route("/api/secrets/envs", get(routes::secrets::list_envs).post(routes::secrets::create_env))
```

---

## T5 — Add unit tests

**File:** `crates/sdlc-server/src/routes/secrets.rs`

Add tests in the existing `#[cfg(test)]` block:
1. `create_env_empty_pairs_returns_bad_request` — verify 400 when `pairs` is empty.
2. `create_env_no_keys_returns_bad_request` — verify 400 when no keys are configured.
3. `create_env_conflict_returns_409` — verify 409 when env already exists (requires `age`; skip if absent).
4. `create_env_success_returns_201` — verify 201 and key_names (requires `age`; skip if absent).

---

## T6 — Build and test

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Fix any build or lint failures before marking complete.
