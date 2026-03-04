# Tasks: Secrets — Add Environment Modal and CLI Hint Affordances

## Task List

### T1: Add `createSecretsEnv` to API client
**File:** `frontend/src/api/client.ts`

Add the `createSecretsEnv` method to the `api` object:

```typescript
createSecretsEnv: (body: { env: string; pairs: { key: string; value: string }[] }) =>
  request('/api/secrets/envs', { method: 'POST', body: JSON.stringify(body) }),
```

Place it after `getSecretsEnvs` and `deleteSecretsEnv` in the secrets section.

---

### T2: Implement `AddEnvModal` component in SecretsPage.tsx
**File:** `frontend/src/pages/SecretsPage.tsx`

Add a new `AddEnvModal` component with the following interface:

```typescript
interface AddEnvModalProps {
  onAdd: (env: string, pairs: { key: string; value: string }[]) => Promise<void>
  onClose: () => void
}
```

State:
- `env: string` — environment name input
- `pairs: { key: string; value: string }[]` — starts with one empty row
- `loading: boolean`
- `error: string | null`

Behavior:
- Submit validates: `env.trim()` non-empty AND at least one pair with `key.trim()` non-empty
- On valid submit: calls `onAdd(env.trim(), pairs.filter(p => p.key.trim()))`, then `onClose()` on success
- On error: sets `error` with the error message (pass through the server message for 409: "already exists")
- "Add row" appends `{ key: '', value: '' }` to pairs
- Trash icon removes a row; disabled when only 1 row remains

Layout follows `AddKeyModal` pattern exactly (fixed overlay, card, X button, error display, spinner on submit).

Required new imports: `Trash2` is already imported.

---

### T3: Wire Add Environment button and modal into SecretsPage
**File:** `frontend/src/pages/SecretsPage.tsx`

In `SecretsPage`:
1. Add `showAddEnv` boolean state (default `false`)
2. Add `handleCreateEnv` async function:
   ```typescript
   const handleCreateEnv = async (env: string, pairs: { key: string; value: string }[]) => {
     await api.createSecretsEnv({ env, pairs })
     refresh()
   }
   ```
3. Render `<AddEnvModal>` (alongside existing `<AddKeyModal>`) when `showAddEnv` is true
4. In the Environments section header, add the "Add Environment" button:
   ```tsx
   <button
     onClick={() => setShowAddEnv(true)}
     className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
   >
     <Plus className="w-3.5 h-3.5" />
     Add Environment
   </button>
   ```
   Place it mirroring the "Add Key" button layout in the Keys section header.

---

### T4: Add CLI set-secret hint to each env card
**File:** `frontend/src/pages/SecretsPage.tsx`

In the env card render (inside the `envs.map(env => ...)` block), add a second hint row below the existing `eval $(sdlc secrets env export ...)` row:

```tsx
{/* Set secret CLI hint */}
<div className="mt-1.5 flex items-center gap-1.5 text-xs text-muted-foreground bg-muted/50 rounded px-2 py-1.5">
  <code className="font-mono flex-1">
    sdlc secrets env set {env.env} KEY=value
  </code>
  <CopyButton text={`sdlc secrets env set ${env.env} KEY=value`} />
</div>
```

This row sits directly after the existing export hint row. Both rows are always visible.

---

## Dependency Order

T1 (API client) → T2 (modal component) → T3 (wire modal) → T4 (env card hint)

T4 is independent of T1-T3 and can be implemented in parallel if desired.
