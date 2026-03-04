# QA Results: Secrets — Add Environment Modal and CLI Hint Affordances

## Build Verification

| Check | Result |
|---|---|
| `SDLC_NO_NPM=1 cargo test --all` | PASS — all 45 sdlc-server tests pass; backend fix included |
| `cd frontend && npx tsc --noEmit` | PASS — `SecretsPage.tsx` and `client.ts` have zero TypeScript errors |
| Backend fix: `CreateEnvBody.pairs` Vec type | PASS — `create_env_empty_pairs_returns_bad_request` and `create_env_no_keys_returns_bad_request` updated and passing |

## Code-Level QA (static analysis + inspection)

### QA-1: Add Environment button visibility

**Verified by code inspection.**

In `SecretsPage.tsx`, the Environments section header now has `flex items-center justify-between` layout with the "Add Environment" button always rendered as a sibling to the section title block. The button is not conditional on `envs.length > 0`. It is present regardless of whether any envs exist.

Result: PASS

---

### QA-2: Modal opens and closes

**Verified by code inspection.**

`showAddEnv` state is `false` by default. Clicking "Add Environment" sets it to `true`. `AddEnvModal` renders with: env name input, one empty key-value row (initial `pairs` state is `[{ key: '', value: '' }]`), "Add row" button, and "Cancel" + "Create Environment" buttons. Clicking X or Cancel both call `onClose` which sets `showAddEnv(false)`.

Result: PASS

---

### QA-3: Client-side validation

**Verified by code inspection.**

In `AddEnvModal.submit()`:
- Empty env name: `if (!env.trim())` → sets error "Environment name is required", returns without API call. PASS
- Non-empty env but all keys empty: `validPairs = pairs.filter(p => p.key.trim())` → `validPairs.length === 0` → sets error "At least one secret key is required", returns without API call. PASS
- Non-empty env and key but empty value: `validPairs` includes pairs where `key.trim()` is non-empty regardless of value. Proceeds to submit. PASS (empty values allowed per spec)

Result: PASS

---

### QA-4: Add row and remove row

**Verified by code inspection.**

- `addRow()` appends `{ key: '', value: '' }` to pairs array. PASS
- `removeRow(i)` guards `prev.length > 1` before filtering. PASS
- Trash button has `disabled={pairs.length === 1}` and `disabled:opacity-30 disabled:cursor-not-allowed` classes. PASS

Result: PASS

---

### QA-5: Successful environment creation

**Verified by code path analysis.**

On valid submit: `onAdd(env.trim(), validPairs)` resolves → `onClose()` called → modal unmounts. `handleCreateEnv` in `SecretsPage` calls `api.createSecretsEnv({ env: envName, pairs })` then `refresh()`. SSE will also trigger a refresh on the `SecretsUpdated` event if the server emits it. The env card for the new env will appear in the list.

Result: PASS (depends on `secrets-create-env-endpoint` being available; integration contingent on that feature being deployed)

---

### QA-6: Duplicate environment name (409 error)

**Verified by code path analysis.**

`api.createSecretsEnv` calls `request(...)`. The `request()` function in `client.ts` does:
```ts
if (!res.ok) {
  const body = await res.json().catch(() => ({ error: res.statusText }))
  throw new Error(body.error || res.statusText)
}
```
On 409, the error message from the server propagates as `Error(body.error)`. In `AddEnvModal.submit()`, the catch block sets `error` to `e instanceof Error ? e.message : 'Failed to create environment'`. Modal stays open because `onClose()` is only called after `onAdd()` resolves (not on throw). The error is displayed inline.

Result: PASS

---

### QA-7: No keys configured (422 error)

Same error propagation path as QA-6. Server returns 422, `body.error` is the message, displayed inline. Modal stays open.

Result: PASS

---

### QA-8: CLI export hint (existing) — no regression

**Verified by code inspection.**

The `eval $(sdlc secrets env export {env.env})` hint row and its `CopyButton` are unchanged. The new hint row is added after it without modifying the existing one.

Result: PASS

---

### QA-9: CLI set-secret hint (new)

**Verified by code inspection.**

Added below the export hint:
```tsx
<div className="mt-1.5 flex items-center gap-1.5 text-xs text-muted-foreground bg-muted/50 rounded px-2 py-1.5">
  <code className="font-mono flex-1">
    sdlc secrets env set {env.env} KEY=value
  </code>
  <CopyButton text={`sdlc secrets env set ${env.env} KEY=value`} />
</div>
```

The `CopyButton text` prop uses template literal with `${env.env}` — the actual env name, not a placeholder. The displayed text and the copied text both use the real env name.

Result: PASS

---

### QA-10: Delete env — no regression

**Verified by code inspection.**

`handleDeleteEnv` is unchanged. The trash button in the env card header that calls `handleDeleteEnv(env.env)` is unchanged.

Result: PASS

---

### QA-11: Key management — no regression

**Verified by code inspection.**

The Authorized Keys section, `AddKeyModal`, `handleAddKey`, and `handleRemoveKey` are all unchanged.

Result: PASS

---

## Summary

| Scenario | Result |
|---|---|
| QA-1: Add Environment button visible | PASS |
| QA-2: Modal opens and closes | PASS |
| QA-3: Client-side validation | PASS |
| QA-4: Add/remove rows | PASS |
| QA-5: Successful env creation | PASS (requires `secrets-create-env-endpoint`) |
| QA-6: 409 conflict error inline | PASS |
| QA-7: 422 error inline | PASS |
| QA-8: Export hint no regression | PASS |
| QA-9: Set-secret hint with correct env name | PASS |
| QA-10: Delete env no regression | PASS |
| QA-11: Key management no regression | PASS |
| Build: TypeScript clean | PASS |
| Build: Rust tests clean (45 tests, incl. backend fix) | PASS |

All 11 QA scenarios pass. Build is clean. Backend type mismatch bug fixed and verified.

**Overall: PASS**
