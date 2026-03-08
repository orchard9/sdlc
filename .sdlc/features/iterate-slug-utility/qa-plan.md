# QA Plan: nextIterationSlug utility

## Test Strategy

Pure unit tests only. No integration or E2E tests needed since this is a stateless utility function with no API or UI dependencies.

## Test Cases

### TC1: Base slug with no existing versions
- Input: `nextIterationSlug('foo', [])`
- Expected: `'foo-v2'`

### TC2: Base slug with one existing version
- Input: `nextIterationSlug('foo', ['foo-v2'])`
- Expected: `'foo-v3'`

### TC3: Base slug with multiple sequential versions
- Input: `nextIterationSlug('foo', ['foo-v2', 'foo-v3', 'foo-v4'])`
- Expected: `'foo-v5'`

### TC4: Versions with gaps
- Input: `nextIterationSlug('foo', ['foo-v2', 'foo-v5'])`
- Expected: `'foo-v6'`

### TC5: Already-versioned input slug
- Input: `nextIterationSlug('foo-v2', ['foo-v2', 'foo-v3'])`
- Expected: `'foo-v4'`

### TC6: Length truncation at 40 characters
- Input: `nextIterationSlug('a-very-long-slug-name-that-exceeds-limit', [])`
- Expected: result is at most 40 characters

### TC7: Unrelated slugs in the existing list are ignored
- Input: `nextIterationSlug('foo', ['bar-v2', 'baz-v3', 'foobar-v2'])`
- Expected: `'foo-v2'`

### TC8: Existing slug list includes the base slug itself (no -vN)
- Input: `nextIterationSlug('foo', ['foo'])`
- Expected: `'foo-v2'` (base without version suffix is not matched by the `-vN` pattern)

## Pass Criteria

All unit tests pass via `npm test -- slug.test`.
