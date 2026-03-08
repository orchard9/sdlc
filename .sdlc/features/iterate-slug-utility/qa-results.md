# QA Results: nextIterationSlug utility

## Test Execution

**Runner:** vitest v4.0.18
**Command:** `npx vitest run src/lib/slug.test.ts`
**Result:** 13/13 tests passed
**Duration:** 518ms

## Test Results

| Test Case | Status |
|---|---|
| titleToSlug: converts a title to a slug | PASS |
| titleToSlug: strips special characters | PASS |
| titleToSlug: collapses multiple hyphens | PASS |
| nextIterationSlug: returns -v2 when no existing versions | PASS |
| nextIterationSlug: returns -v3 when v2 exists | PASS |
| nextIterationSlug: returns -v5 when v2, v3, v4 exist | PASS |
| nextIterationSlug: skips gaps and returns max+1 | PASS |
| nextIterationSlug: strips -vN from input slug before computing | PASS |
| nextIterationSlug: ignores unrelated slugs | PASS |
| nextIterationSlug: ignores the base slug without a version suffix | PASS |
| nextIterationSlug: truncates result to 40 characters | PASS |
| nextIterationSlug: handles hyphenated base slugs correctly | PASS |
| nextIterationSlug: handles high version numbers | PASS |

## Verdict

All QA plan test cases covered and passing. Ready for merge.
