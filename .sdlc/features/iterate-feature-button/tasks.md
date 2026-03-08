# Tasks: Iterate Button on FeatureDetail

## T1: Create nextIterationSlug utility
Create `frontend/src/lib/iterateSlug.ts` with the `nextIterationSlug(baseSlug: string, existingSlugs: string[]): string` function. Logic: strip any existing `-vN` suffix, find the highest existing version, return `baseSlug-v{N+1}`. Default to `-v2` if no versions exist.

## T2: Add Iterate button to FeatureDetail released banner
In `FeatureDetail.tsx`, add the "Iterate" button inside the released banner. Include state for `iterating` loading flag, fetch ponder slugs on mount, wire up `handleIterate` to call `nextIterationSlug` + `api.createPonderEntry` + `navigate`.

## T3: Add unit tests for nextIterationSlug
Create `frontend/src/lib/iterateSlug.test.ts` with tests covering: base slug with no versions, incrementing existing versions, collision avoidance, and edge cases (e.g., slug ending in `-v` without a number).
