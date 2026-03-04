# Design: Secrets — Add Environment Modal and CLI Hint Affordances

## Overview

This is a pure frontend change to `frontend/src/pages/SecretsPage.tsx`. It adds:

1. **Add Environment modal** — allows users to create a new encrypted env file from the UI, calling `POST /api/secrets/envs` (delivered by `secrets-create-env-endpoint`).
2. **CLI set-secret hint** — a copy-button hint row on each existing environment card showing `sdlc secrets env set <env> KEY=value`.

No server-side changes. No new backend routes. No new type definitions (the existing `SecretsEnvMeta` type is sufficient).

## Component Architecture

All changes live in `SecretsPage.tsx`. Two new components are added alongside existing ones:

```
SecretsPage.tsx
├── CopyButton          (existing — reused as-is)
├── AddKeyModal         (existing — no changes)
├── AddEnvModal         (NEW — env creation modal)
└── SecretsPage         (existing — extended)
    ├── Authorized Keys section  (existing — no changes)
    └── Environments section     (extended)
        ├── Header: "Add Environment" button (NEW)
        ├── Empty state: existing CLI hint (existing — no changes)
        └── Env cards: + CLI set-secret hint row (NEW)
```

## Wireframes

### Environments Section Header (with Add button)

```
┌─────────────────────────────────────────────────────────────┐
│  🔒  Environments  [2]                    + Add Environment  │
├─────────────────────────────────────────────────────────────┤
│  ...env cards...                                             │
└─────────────────────────────────────────────────────────────┘
```

The "Add Environment" button mirrors the existing "Add Key" button styling:
- `text-xs text-muted-foreground hover:text-foreground transition-colors`
- `Plus` icon (3.5×3.5) + "Add Environment" text

### Add Environment Modal

```
┌───────────────────────────────────────────────────────────┐
│  Add Environment                                        [X] │
├───────────────────────────────────────────────────────────┤
│  Environment name                                          │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ e.g. production, staging                            │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                            │
│  Secrets                                                   │
│  ┌─────────────────────────┐  ┌───────────────────────┐   │
│  │ KEY (e.g. API_KEY)      │  │ value                 │ 🗑 │
│  └─────────────────────────┘  └───────────────────────┘   │
│  ┌─────────────────────────┐  ┌───────────────────────┐   │
│  │ KEY                     │  │ value                 │ 🗑 │
│  └─────────────────────────┘  └───────────────────────┘   │
│  + Add row                                                 │
│                                                            │
│  ⚠ An environment named 'production' already exists.      │
│                                                            │
│                               [Cancel]  [Create Environment]│
└───────────────────────────────────────────────────────────┘
```

**Modal fields:**
- `env` text input: env name (placeholder "e.g. production, staging")
- Key-value list: rows of `[KEY input] [VALUE input] [trash button]`
  - Minimum 1 row, pre-populated empty on open
  - Trash disabled when only 1 row remains
  - KEY input: monospace, placeholder `MY_API_KEY`
  - VALUE input: monospace, placeholder `value`
- "Add row" button: small muted text link below list
- Error banner: shown on API errors (409 or other)
- Submit button: "Create Environment", disabled while loading

### Env Card — CLI Set-Secret Hint (new row)

```
┌───────────────────────────────────────────────────────────┐
│  production                                            🗑  │
│  2 keys · updated 3/1/2026                               │
│  [jordan]  [ci-bot]                                      │
│  ┌──────────────────────────────────────────────────┐    │
│  │ eval $(sdlc secrets env export production)   [📋]│    │
│  └──────────────────────────────────────────────────┘    │
│  ┌──────────────────────────────────────────────────┐    │
│  │ sdlc secrets env set production KEY=value    [📋]│    │  ← NEW
│  └──────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────┘
```

The new hint row matches the existing `eval $(...)` row exactly in styling:
- Same `bg-muted/50 rounded px-2 py-1.5` container
- Same `text-xs text-muted-foreground font-mono` text
- Same `CopyButton` component

## Data Flow

```
User fills modal → clicks "Create Environment"
  → api.createSecretsEnv({ env, pairs })
  → POST /api/secrets/envs
  → 201 Created: modal closes; SSE fires SecretsUpdated → env list refreshes
  → 409 Conflict: inline error "already exists"
  → 422 Unprocessable: inline error (no keys configured, or bad input)
  → 5xx: inline error with message
```

SSE handles refresh — no explicit `refresh()` call after success (the SSE subscription in `SecretsPage` already calls `refresh` on relevant events).

## API Client Addition

```typescript
// frontend/src/api/client.ts
createSecretsEnv: (body: { env: string; pairs: { key: string; value: string }[] }) =>
  request('/api/secrets/envs', { method: 'POST', body: JSON.stringify(body) }),
```

## State Shape (AddEnvModal)

```typescript
interface Pair { key: string; value: string }

// Local state inside AddEnvModal
const [env, setEnv] = useState('')
const [pairs, setPairs] = useState<Pair[]>([{ key: '', value: '' }])
const [loading, setLoading] = useState(false)
const [error, setError] = useState<string | null>(null)
```

## Validation

Client-side validation before submit:
- `env.trim()` must be non-empty
- At least one pair where `key.trim()` is non-empty (value can be empty)
- If validation fails: show inline error, do not call API

## Implementation Notes

- `AddEnvModal` follows the exact same pattern as `AddKeyModal` for consistency: fixed overlay, card container, X button, error display, loading spinner on submit button.
- The `SecretsPage` state gains `showAddEnv: boolean` alongside the existing `showAddKey: boolean`.
- The `handleCreateEnv` function in `SecretsPage` calls `api.createSecretsEnv(...)` and closes the modal on success; errors are re-thrown to the modal for inline display.
- No changes to existing `handleAddKey`, `handleRemoveKey`, `handleDeleteEnv` functions.

## Files Changed

| File | Change |
|---|---|
| `frontend/src/pages/SecretsPage.tsx` | Add `AddEnvModal` component; extend `SecretsPage` with Add button + modal + env card hint |
| `frontend/src/api/client.ts` | Add `createSecretsEnv` method |
