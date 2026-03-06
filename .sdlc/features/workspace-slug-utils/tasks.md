# Tasks: workspace-slug-utils

## T1: Create frontend/src/lib/slug.ts with exported titleToSlug

Create the new shared utility file. Export `titleToSlug` with the exact same implementation as the existing copies.

## T2: Update NewIdeaModal.tsx to import from lib/slug

Remove the local `function titleToSlug` definition. Add `import { titleToSlug } from '../../lib/slug'`.

## T3: Update InvestigationPage.tsx to import from lib/slug

Remove the local `function titleToSlug` definition. Add `import { titleToSlug } from '../lib/slug'`.

## T4: Update PonderPage.tsx to import from lib/slug

Remove the local `function titleToSlug` definition. Add `import { titleToSlug } from '../lib/slug'`.

## T5: Update GuidelinePage.tsx to import from lib/slug

Remove the local `function titleToSlug` definition. Add `import { titleToSlug } from '../lib/slug'`.

## T6: Update EvolvePage.tsx to import from lib/slug

Remove the local `function titleToSlug` definition. Add `import { titleToSlug } from '../lib/slug'`.

## T7: Verify build passes

Run `npm run build` in `frontend/` and confirm no TypeScript errors. Confirm `grep -r "function titleToSlug" frontend/src` returns no results.
