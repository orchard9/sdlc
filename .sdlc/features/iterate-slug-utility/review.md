# Code Review: nextIterationSlug utility

## Files Changed

| File | Change |
|---|---|
| `frontend/src/lib/slug.ts` | Added `escapeRegex` (private) and `nextIterationSlug` (exported) |
| `frontend/src/lib/slug.test.ts` | New file — 13 unit tests for both `titleToSlug` and `nextIterationSlug` |

## Findings

### 1. Correctness
- The version-stripping regex (`/-v\d+$/`) correctly handles both versioned and unversioned input slugs.
- `escapeRegex` defensively prevents regex injection from slug characters.
- The `maxVersion` approach correctly handles gaps (v2, v5 yields v6).
- Length cap at 40 chars matches system conventions.
- All 13 tests pass.

**Verdict:** No issues found.

### 2. Code Quality
- Function is pure with no side effects — easy to test and reason about.
- JSDoc with examples documents behavior clearly.
- `escapeRegex` is kept private (not exported) since it is an implementation detail.
- Colocated with `titleToSlug` in `slug.ts` — good module cohesion.

**Verdict:** Clean, follows project conventions.

### 3. Test Coverage
- Tests cover: no versions, sequential versions, gaps, already-versioned input, unrelated slugs, high version numbers, length truncation, hyphenated slugs.
- Also includes tests for the pre-existing `titleToSlug` function (bonus coverage).

**Verdict:** Comprehensive coverage for a utility function.

## Summary

No issues to fix. The implementation is minimal, correct, well-tested, and follows project conventions.
