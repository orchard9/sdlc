# Code Review: Secrets — Add Environment Modal and CLI Hint Affordances

## Summary

Three files changed.

- `frontend/src/api/client.ts` — 3 lines added (`createSecretsEnv` method)
- `frontend/src/pages/SecretsPage.tsx` — ~135 lines added (`AddEnvModal` component, state wiring, UI changes)
- `crates/sdlc-server/src/routes/secrets.rs` — Bug fix: `CreateEnvBody.pairs` changed from `HashMap<String, String>` to `Vec<EnvPair>` to match the frontend's array-of-objects format.

No new dependencies.

## Review

### Correctness

**FR-1: Add Environment button** — Present in the Environments section header. Consistent styling with the Authorized Keys "Add Key" button (`flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground`). Button is always visible regardless of env count. ✓

**FR-2: AddEnvModal** — Follows `AddKeyModal` pattern exactly: fixed overlay, card container, X close button, error display, loading spinner on submit. State shape is minimal and correct (`env`, `pairs`, `loading`, `error`). Validation correctly requires non-empty env name and at least one pair with a non-empty key (values may be empty). On success: `onAdd()` resolves → `onClose()` called. On error: error message is set and displayed inline; modal stays open. Trash button is disabled (`disabled:cursor-not-allowed disabled:opacity-30`) when only 1 row remains, preventing the user from removing the last pair. ✓

**FR-3: CLI set-secret hint** — Added below the export hint on each env card. Same container styling (`bg-muted/50 rounded px-2 py-1.5`), same `CopyButton` component, same monospace text. The copy text uses template literal interpolation with the actual env name, not a placeholder. ✓

**FR-4: API client method** — `createSecretsEnv` added in the correct position (after `deleteSecretsEnv`, within the secrets section). Type matches spec: `{ env: string; pairs: { key: string; value: string }[] }`. ✓

**SSE refresh** — `handleCreateEnv` calls `refresh()` after `api.createSecretsEnv` resolves. SSE subscription via `useSSE(refresh)` will additionally pick up server-side events. No double-loading issue since `refresh` is idempotent. ✓

### Consistency

- `AddEnvModal` uses identical visual structure to `AddKeyModal`: overlay → card → header with X → body → footer with Cancel + submit.
- Error display pattern is the same (`AlertCircle` icon + destructive text).
- Loading spinner pattern is the same (`Loader2` + `animate-spin`).
- The Pair interface is declared at file scope (not inside the component), which is correct TypeScript convention.
- `updatePair` uses a computed property key (`{ ...p, [field]: val }`) — clean immutable update.

### Edge Cases

- **Empty value rows**: Only pairs where `key.trim()` is non-empty are sent. Rows with empty keys are filtered out before submission. This means a user can add rows with only values and they'll be silently dropped — acceptable behavior for this feature scope.
- **Duplicate env name (409)**: The server error message propagates through `api.createSecretsEnv` → thrown as `Error(body.error)` by the `request()` function → caught in modal's `submit()` → displayed inline. The 409 message from the server is "environment already exists" — this will display correctly.
- **Remove row guard**: `removeRow` checks `prev.length > 1` before filtering. The trash button is also `disabled` when `pairs.length === 1`, so both UI and logic guard against removing the last row.

### No Regressions

- `handleAddKey`, `handleRemoveKey`, `handleDeleteEnv` unchanged.
- The `AddKeyModal` component is unchanged.
- The Environments section empty state is unchanged.
- SSE subscription pattern (`useSSE(refresh)`) is unchanged.
- TypeScript compilation of changed files: clean (verified with `tsc -b`).
- Rust test suite: all passing (verified with `SDLC_NO_NPM=1 cargo test --all`).

### Critical Bug Fixed During Review

**Backend `CreateEnvBody.pairs` type mismatch:** The `secrets-create-env-endpoint` feature implemented `pairs` as `HashMap<String, String>` expecting `{ "KEY": "VALUE" }` JSON object, but the `secrets-env-ui` spec and frontend implementation use `[{ key, value }]` array format. This would have caused a JSON deserialization error (422) on every env creation attempt.

**Fix applied:**
- Added `EnvPair { key: String, value: String }` struct
- Changed `CreateEnvBody.pairs` to `Vec<EnvPair>`
- Updated `pairs_content` serialization: `.map(|p| format!("{}={}", p.key, p.value))`
- Updated unit tests to use `vec![]` and `vec![EnvPair { ... }]`
- All 45 `sdlc-server` tests pass after fix.

### Minor Observations (non-blocking)

1. **Array index as `key` in pairs map** (`key={i}`) — acceptable for a short-lived local list that doesn't reorder. For this modal, indices are stable enough that this doesn't cause display bugs.
2. **`handleCreateEnv` does not set `actionError`** — errors from env creation are handled inside the modal's own `error` state rather than the page-level `actionError`. This is the correct pattern: the modal owns its error display so it stays open on failure with the error visible.
3. **`refresh()` called explicitly** in `handleCreateEnv` as a belt-and-suspenders approach alongside SSE. This matches the pattern used in `handleAddKey`. No concern.

## Verdict

Implementation is complete and correct. One critical type mismatch bug was found and fixed during this review. All acceptance criteria met.
