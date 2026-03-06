# Design: workspace-slug-utils

## Overview

This is a pure refactoring — no new behavior, no UI changes, no architectural shifts. The design is straightforward: extract the duplicated `titleToSlug` function into a shared module and update all import sites.

## Target Location

`frontend/src/lib/slug.ts`

The `lib/` directory already holds shared utilities (`types.ts`, `utils.ts`, `phases.ts`, etc.), making it the natural home for this function.

## Module Shape

```ts
// frontend/src/lib/slug.ts

/**
 * Converts a human-readable title string into a URL-safe slug.
 * Lowercases, strips non-alphanumeric characters, collapses whitespace
 * and hyphens, and trims leading/trailing hyphens.
 */
export function titleToSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}
```

## Call Sites to Update

| File | Import path |
|---|---|
| `frontend/src/components/ponder/NewIdeaModal.tsx` | `../../lib/slug` |
| `frontend/src/pages/InvestigationPage.tsx` | `../lib/slug` |
| `frontend/src/pages/PonderPage.tsx` | `../lib/slug` |
| `frontend/src/pages/GuidelinePage.tsx` | `../lib/slug` |
| `frontend/src/pages/EvolvePage.tsx` | `../lib/slug` |

Each file: add an `import { titleToSlug } from '<path>'` line, delete the local `function titleToSlug` block.

## Verification

- `npm run build` (in `frontend/`) passes with no TypeScript errors.
- `grep -r "function titleToSlug" frontend/src` returns no results after the change.
