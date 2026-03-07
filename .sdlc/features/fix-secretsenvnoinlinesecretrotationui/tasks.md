# Tasks: Inline Secret Rotation UI

## Task 1: Add `PATCH /api/secrets/envs/:name` backend route

**File**: `crates/sdlc-server/src/routes/secrets.rs`

- Add `UpdateEnvBody` struct (reuses `EnvPair`)
- Add `update_env` handler:
  - 400 if `pairs` is empty
  - 404 if env file does not exist
  - 400 if no recipient keys configured
  - Calls `secrets::write_env()` to re-encrypt with new pairs
  - Returns `{ status: "updated", env: name, key_names: [...] }`
- Add unit tests: `patch_env_not_found_returns_404`, `patch_env_empty_pairs_returns_bad_request`, `patch_env_no_keys_returns_bad_request`

## Task 2: Register `PATCH` route in router

**File**: `crates/sdlc-server/src/lib.rs`

- Add `.route("/api/secrets/envs/:name", patch(secrets::update_env))` to the router

## Task 3: Add `updateSecretsEnv` to API client

**File**: `frontend/src/api/client.ts`

- Add `updateSecretsEnv(name: string, pairs: { key: string; value: string }[])` method calling `PATCH /api/secrets/envs/:name`

## Task 4: Add `EditEnvModal` component and Edit button to `SecretsPage.tsx`

**File**: `frontend/src/pages/SecretsPage.tsx`

- Add `EditEnvModal` component:
  - Accepts `env: SecretsEnvMeta` and `onUpdate`/`onClose` props
  - Pre-populates key fields from `env.key_names` with empty value fields
  - Warning banner explaining replace semantics
  - Add/remove row buttons
  - Filters out blank-value pairs before submit
  - Error display on failure
- Add `Pencil` icon import from `lucide-react`
- Add `showEditEnv: SecretsEnvMeta | null` state to `SecretsPage`
- Add Edit (✏) button alongside Trash2 in each env card header
- Add `handleUpdateEnv` handler calling `api.updateSecretsEnv()` then `refresh()`
- Render `EditEnvModal` conditionally when `showEditEnv` is set
