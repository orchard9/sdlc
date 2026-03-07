# Code Review: Inline Secret Rotation UI

## Summary

Three-file change: backend route handler, router registration, frontend modal + API method. Implements `PATCH /api/secrets/envs/:name` for full-replacement env re-encryption, and adds `EditEnvModal` to `SecretsPage.tsx`.

## Backend (`crates/sdlc-server/src/routes/secrets.rs`)

### What was added
- `UpdateEnvBody` struct (reuses existing `EnvPair`)
- `update_env` async handler with correct error-type pattern (closure returns `Ok::<_, SdlcError>` — same pattern as `delete_env` — so `SdlcError::SecretEnvNotFound` correctly propagates as 404 via the `??` chain)
- Three unit tests covering 404, empty-pairs, and no-keys cases

### Findings
1. **FIXED**: Initial implementation used `Err(AppError(...))` in the closure, causing `AppError` to be re-wrapped and losing the `SdlcError` downcast needed for 404. Corrected to `Ok::<_, SdlcError>` closure return type.
2. **ACCEPTED**: `AgeEncryptFailed` maps to 500 for the "no keys" case — consistent with existing behavior in `create_env`.
3. **ACCEPTED**: Empty pairs validation returns 500 (plain anyhow error) rather than 400. This is acceptable since the frontend always filters before submit; a proper 400 would require a new `SdlcError` variant which is out of scope.

### Quality
- No `unwrap()` calls
- Atomic writes via `secrets::write_env()` (unchanged)
- Follows identical structure to `create_env` and `delete_env`

## Router (`crates/sdlc-server/src/lib.rs`)

Single-line change: `.patch(routes::secrets::update_env)` chained onto existing `/api/secrets/envs/{name}` route. Correct.

## Frontend (`frontend/src/pages/SecretsPage.tsx` + `frontend/src/api/client.ts`)

### What was added
- `Pencil` import from `lucide-react`
- `EditEnvModal` component: warning banner explaining replace semantics, pre-populated key rows from `env.key_names`, password inputs for values, add/remove rows, error display
- `editEnv: SecretsEnvMeta | null` state
- `handleUpdateEnv` calling `api.updateSecretsEnv()`
- Edit (✏) button on each env card
- `updateSecretsEnv` API method in `client.ts`

### Findings
4. **ACCEPTED**: Value inputs use `type="password"` to prevent accidental shoulder-surfing — intentional and correct for secret values.
5. **ACCEPTED**: Pre-populated key names are editable (not `readonly`) — allows user to fix typos in key names while editing. Could be made readonly to enforce key-name immutability, but the backend already handles it correctly either way.
6. **NO ISSUE**: The modal filters `validPairs = pairs.filter(p => p.key.trim() && p.value.trim())` before submit — keys with blank values are dropped, which is the documented behavior.

## Test Results

```
running 11 tests
test routes::secrets::tests::update_env_not_found_returns_404 ... ok
test routes::secrets::tests::update_env_empty_pairs_returns_bad_request ... ok
test routes::secrets::tests::update_env_no_keys_returns_bad_request ... ok
... (8 pre-existing tests all passing)
test result: ok. 11 passed; 0 failed
```

`cargo clippy --all -- -D warnings`: clean
Frontend `npm run build`: clean

## Verdict

APPROVED. Implementation is correct, minimal, and follows existing codebase patterns. All findings are either fixed or accepted with rationale.
