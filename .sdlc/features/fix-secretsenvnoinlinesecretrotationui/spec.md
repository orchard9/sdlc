# Spec: Inline Secret Rotation UI for Environments

## Problem

The `/secrets` page lets users create and delete environments, but provides no UI to update or rotate individual secret values within an existing environment. The only affordance is a CLI hint (`sdlc secrets env set <env> KEY=value`). Rotating a compromised or expired secret requires CLI access, defeating the web UI's purpose.

**Root cause (dual gap):**
1. **Backend**: No `PATCH /api/secrets/envs/:name` endpoint — only `POST` (create) and `DELETE` (destroy entire env). The `set_env_pairs()` core function exists but is not exposed via HTTP.
2. **Frontend**: `SecretsPage.tsx` renders key names as read-only badges with no edit/rotate affordance per key or per env.

**Security constraint**: The server never handles plaintext secrets — encrypted `.age` files require the user's private key to decrypt. Server-side decryption is intentionally not done. This means any "update" operation must work with full-replacement semantics: the user provides ALL desired key-value pairs, the server re-encrypts with those pairs.

## Proposed Solution

### Backend: `PATCH /api/secrets/envs/:name`

Add a new route that accepts a full replacement set of key-value pairs and re-encrypts the env file:

- **Request**: `PATCH /api/secrets/envs/:name` → `{ pairs: [{ key, value }] }`
- **Validation**: 404 if env doesn't exist, 400 if pairs is empty or no keys configured
- **Behavior**: Calls `secrets::write_env()` to re-encrypt with the new pairs (replace semantics, not merge — server can't decrypt existing content)
- **Response**: `{ status: "updated", env: name, key_names: [...] }`

### Frontend: "Edit Environment" modal

Add an **Edit** button per environment card. Clicking opens a modal that:

1. Shows all existing key names (from `env.key_names`) as pre-populated key input rows with empty value fields
2. Allows adding new rows (for new keys) and removing rows (to delete a key)
3. Displays clear explanatory copy: *"Enter values for each key and submit to re-encrypt this environment. Keys with no value entered will be removed. You must provide values for all secrets you want to keep."*
4. On submit: calls `PATCH /api/secrets/envs/:name` with all non-empty pairs, then refreshes

**Per-key "Set" inline affordance**: Each key badge in the env card gets a rotate/edit icon. Clicking opens a compact inline form (or the Edit modal pre-focused on that key) so the user understands they need to fill ALL values.

## Acceptance Criteria

1. `PATCH /api/secrets/envs/:name` is registered in the router and returns 404 for missing envs, 400 for empty/no-keys cases, and 200 on success
2. `GET /api/secrets/envs` after a PATCH reflects updated `key_names` and `updated_at`
3. The `/secrets` page renders an Edit button on each environment card
4. Clicking Edit opens a modal with all current key names pre-populated as rows
5. Submitting the modal with valid values calls PATCH and refreshes the env list
6. The modal clearly communicates replace semantics (keys not filled in are removed)
7. Error states (backend failures, validation) are surfaced in the modal

## Out of Scope

- Server-side decryption or merge semantics (security boundary — server never holds plaintext)
- Decryption in the browser (no private key access)
- Individual per-key PATCH with true merge (requires decryption)
- Audit logging of which keys were changed

## Files to Change

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/secrets.rs` | Add `patch_env` handler |
| `crates/sdlc-server/src/lib.rs` | Register `PATCH /api/secrets/envs/:name` route |
| `frontend/src/pages/SecretsPage.tsx` | Add `EditEnvModal` component + Edit button per card |
| `frontend/src/api/client.ts` | Add `updateSecretsEnv()` API method |
