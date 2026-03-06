# Spec: workspace-slug-utils

## Problem

The `titleToSlug` function is copy-pasted verbatim in 5 separate files:

- `frontend/src/components/ponder/NewIdeaModal.tsx`
- `frontend/src/pages/InvestigationPage.tsx`
- `frontend/src/pages/PonderPage.tsx`
- `frontend/src/pages/GuidelinePage.tsx`
- `frontend/src/pages/EvolvePage.tsx`

All 5 copies are identical:
```ts
function titleToSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}
```

This duplication means a bug fix or behavioral change requires updating 5 files, and drift between copies becomes a risk as the codebase grows.

## Goal

Extract `titleToSlug` into a shared utility module at `frontend/src/lib/slug.ts` and update all 5 call sites to import it from there. No behavioral changes — this is a pure refactor.

## Scope

- **In scope**: Create `frontend/src/lib/slug.ts`, export `titleToSlug`, update the 5 files to import from it, remove the local function definitions.
- **Out of scope**: Changing the slug algorithm, adding new utility functions, renaming the function.

## Acceptance Criteria

1. `frontend/src/lib/slug.ts` exists and exports `titleToSlug`.
2. All 5 source files import `titleToSlug` from `../../lib/slug` (or the correct relative path) rather than defining it locally.
3. No local `function titleToSlug` remains in any of the 5 files.
4. The slug algorithm is unchanged — same implementation, same behavior.
5. The frontend builds without TypeScript errors (`npm run build`).
6. No runtime regressions — slug generation in ponder, investigation, evolve, and guideline flows remains identical.
