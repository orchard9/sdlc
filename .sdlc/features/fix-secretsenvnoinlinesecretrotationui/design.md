# Design: Inline Secret Rotation UI

## Architecture

Three-layer change: backend route → API client → frontend modal.

```
PATCH /api/secrets/envs/:name
  └─ secrets::write_env(root, name, pairs_content, &config.keys)
       └─ age encrypt → .sdlc/secrets/envs/<name>.age
       └─ save_env_meta → .sdlc/secrets/envs/<name>.meta.yaml
```

### Security model (unchanged)

The server holds only public keys (recipients). `write_env()` passes all key-value pairs to the `age` binary for encryption. No plaintext is ever stored. PATCH semantics are **full replacement** — the submitted pairs become the complete new content of the encrypted file. Keys not submitted are removed.

---

## Backend: `PATCH /api/secrets/envs/:name`

**File**: `crates/sdlc-server/src/routes/secrets.rs`

```rust
#[derive(serde::Deserialize)]
pub struct UpdateEnvBody {
    pub pairs: Vec<EnvPair>,  // reuse existing EnvPair struct
}

/// PATCH /api/secrets/envs/:name — replace secrets in an existing env file.
pub async fn update_env(
    State(app): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateEnvBody>,
) -> Result<Json<serde_json::Value>, AppError> { ... }
```

**Validation**:
- 400 if `pairs` is empty
- 404 if `.<name>.age` does not exist (use `SecretEnvNotFound`)
- 400 if no keys configured

**Router registration** (`crates/sdlc-server/src/lib.rs`):
```rust
.route("/api/secrets/envs/:name", patch(secrets::update_env))
```

---

## API Client

**File**: `frontend/src/api/client.ts`

```ts
updateSecretsEnv: (name: string, pairs: { key: string; value: string }[]) =>
  request(`/api/secrets/envs/${encodeURIComponent(name)}`, {
    method: 'PATCH',
    body: JSON.stringify({ pairs }),
  }),
```

---

## Frontend: `EditEnvModal`

**File**: `frontend/src/pages/SecretsPage.tsx`

A new modal component modeled after `AddEnvModal`. Differences:
- No env name field (env name is fixed)
- Key fields pre-populated from `env.key_names`; value fields empty (server can't decrypt)
- User must fill in values for all keys they want to retain
- Copy: "Enter values for each key. Keys left blank will be removed. Submit re-encrypts this environment."
- Add/remove row buttons for new keys or key deletion

**Env card update**: Add Edit (`Pencil` icon) button alongside the existing `Trash2` delete button.

```
┌─────────────────────────────────────────────────────┐
│ production                                    ✏  🗑 │
│ 3 keys · updated 3/7/2026                           │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ │
│ │ API_KEY      │ │ DB_PASSWORD  │ │ WEBHOOK_SEC  │ │
│ └──────────────┘ └──────────────┘ └──────────────┘ │
│ eval $(sdlc secrets env export production)    [copy] │
│ sdlc secrets env set production KEY=value     [copy] │
└─────────────────────────────────────────────────────┘
```

**Edit modal layout**:

```
┌─────────────────────────────────────────────────────┐
│ Edit Environment: production                      ✕ │
├─────────────────────────────────────────────────────┤
│ ⚠ Enter values for all keys you want to keep.      │
│   Keys left blank will be removed from this env.    │
│   Submit re-encrypts the environment.               │
├─────────────────────────────────────────────────────┤
│ KEY              │ VALUE                      │  🗑  │
│ API_KEY          │ [__________________]       │  ✕  │
│ DB_PASSWORD      │ [__________________]       │  ✕  │
│ WEBHOOK_SECRET   │ [__________________]       │  ✕  │
│ + Add row                                           │
├─────────────────────────────────────────────────────┤
│                         [Cancel]  [Update Secrets]  │
└─────────────────────────────────────────────────────┘
```

---

## UI Mockup

[Mockup](mockup.html)

---

## Data Flow

1. User clicks ✏ Edit on env card
2. `EditEnvModal` opens with `initialKeys = env.key_names`
3. User fills in values (and optionally adds/removes keys)
4. Submit → filter out blank-value pairs → call `api.updateSecretsEnv(env.env, pairs)`
5. On success: close modal, call `refresh()`
6. On error: display inline error in modal

## Tests to Add

- `patch_env_not_found_returns_404` — PATCH to non-existent env
- `patch_env_empty_pairs_returns_bad_request` — empty pairs validation
- `patch_env_no_keys_returns_bad_request` — no configured recipients
