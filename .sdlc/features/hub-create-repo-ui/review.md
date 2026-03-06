# Code Review: hub-create-repo-ui

## Changes

- `frontend/src/lib/types.ts` — added `CreateRepoResponse` interface
- `frontend/src/api/client.ts` — added `createRepo(name)` function
- `frontend/src/pages/HubPage.tsx` — added `Copy`, `Plus` icon imports; `CreateRepoResponse` type import; `CopyButton` component; `CreateRepoSection` component; "Add New Project" section in fleet view; fixed pre-existing `instance.name` → `instance.slug` bug in `onFleetProvisioned`

## Findings

### PASS — Type safety

`CreateRepoResponse` is fully typed. `api.createRepo` return is inferred via `import()` inline type reference (consistent with existing client.ts pattern). No `any` casts.

### PASS — State machine correctness

State transitions: `idle → creating → done | error`. On error, typing new input resets state to `idle`. "Add another project" resets all state. No stuck states.

### PASS — Copy UX

`CopyButton` uses `navigator.clipboard.writeText`, catches failure silently (no user-visible error for clipboard permission issues — acceptable for a developer tool). "Copied!" transient state lasts 1.5s. Resets correctly.

### PASS — Name validation

Client-side regex `/^[a-z0-9][a-z0-9-]*$/` matches server-side validation. Error shown inline on first invalid character, not on submit. Create button disabled when `nameError` is set.

### PASS — Error messages

409 conflict surfaced as user-friendly "A repo named X already exists" message. Generic API errors shown as-is. Both rendered with `AlertCircle` icon consistent with `ImportSection`.

### PASS — Build

`npm run build` — clean, no TypeScript errors.

### INCIDENTAL FIX — `instance.name` → `instance.slug`

Pre-existing TypeScript error in `onFleetProvisioned`: `instance.name` was used where `instance.slug` is correct (per `FleetInstance` interface). Fixed as part of this change. The build was already failing on this line before my changes.

## Verdict

APPROVED.
