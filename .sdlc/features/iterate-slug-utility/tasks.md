# Tasks: nextIterationSlug utility

## Task 1: Implement nextIterationSlug function
Add the `nextIterationSlug` function to `frontend/src/lib/slug.ts`. The function strips trailing `-vN` from the input slug to find the base, scans existing slugs for version matches, and returns `{base}-v{max+1}` capped at 40 characters.

## Task 2: Add unit tests
Create `frontend/src/lib/slug.test.ts` with tests covering: no existing versions, sequential versions, gap versions, already-versioned input, and length truncation.
