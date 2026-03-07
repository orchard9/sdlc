# QA Plan: Inline Secret Rotation UI

## Unit Tests (Rust)

| Test | File | Assertion |
|---|---|---|
| `patch_env_not_found_returns_404` | `routes/secrets.rs` | PATCH to non-existent env → 404 |
| `patch_env_empty_pairs_returns_bad_request` | `routes/secrets.rs` | PATCH with `pairs: []` → 400 |
| `patch_env_no_keys_returns_bad_request` | `routes/secrets.rs` | PATCH with no configured recipient keys → 400 |

Run: `SDLC_NO_NPM=1 cargo test -p sdlc-server secrets -- --nocapture`

## Integration Check (Rust)

- `cargo clippy --all -- -D warnings` passes with no new warnings

## Manual Smoke Test (local dev)

1. Start server: `cargo run -p sdlc-cli -- serve --port 7777`
2. Navigate to `http://localhost:7777/secrets`
3. Add a recipient key (ssh or age)
4. Create an env `staging` with keys `API_KEY=old` and `DB_PASS=old`
5. Verify: env card shows both key names + new ✏ Edit button
6. Click Edit → modal opens pre-populated with `API_KEY` and `DB_PASS` (values blank)
7. Enter new values → click "Update Secrets" → modal closes, env card shows updated timestamp
8. Run `eval $(sdlc secrets env export staging)` → confirm new values are live
9. Click Edit again → enter only `API_KEY=newer` (blank `DB_PASS`) → submit → `DB_PASS` is removed
10. Verify `key_names` in env card reflects removal

## Error Cases

| Scenario | Expected |
|---|---|
| PATCH with no pairs submitted | Backend 400, frontend shows inline error |
| PATCH to deleted env | Backend 404, frontend shows inline error |
| `age` binary not installed | Backend 500 propagated, frontend shows error |
| Cancel modal | No change, env list unchanged |

## Frontend (visual)

- Edit button (✏) is present on every env card
- Trash2 (delete) button remains and still works
- Edit modal: warning banner is visible and legible
- Edit modal: keys pre-populated as read-only, values editable
- Edit modal: "Add new key" adds a row with editable key and value
- Edit modal: ✕ per row removes that row
- After successful update: env card `updated_at` reflects new timestamp (via SSE refresh)
