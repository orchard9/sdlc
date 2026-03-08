# Spec: nextIterationSlug utility

## Overview

A pure utility function that computes the next versioned iteration slug from a base slug, checking existing slugs for collisions. This supports the "Iterate" workflow where released milestones and features can spawn follow-up ponder entries with auto-incremented slugs.

## Function Signature

```typescript
export function nextIterationSlug(baseSlug: string, existingSlugs: string[]): string
```

## Behavior

1. **Strip version suffix**: Remove any trailing `-vN` from the input slug to find the base (e.g., `git-status-indicator-v2` becomes `git-status-indicator`).
2. **Scan for existing versions**: Search `existingSlugs` for entries matching `{base}-vN` pattern. The original (unversioned) slug is implicitly version 1.
3. **Return next version**: Return `{base}-v{maxVersion + 1}`.
4. **Length cap**: Truncate the result to 40 characters to respect slug length limits.

## Examples

| Input slug | Existing slugs | Output |
|---|---|---|
| `git-status-indicator` | `[]` | `git-status-indicator-v2` |
| `git-status-indicator` | `['git-status-indicator-v2']` | `git-status-indicator-v3` |
| `git-status-indicator-v2` | `['git-status-indicator-v2', 'git-status-indicator-v3']` | `git-status-indicator-v4` |
| `foo` | `['foo-v2', 'foo-v3', 'foo-v5']` | `foo-v6` |

## Location

Add `nextIterationSlug` to the existing `frontend/src/lib/slug.ts` file, which already contains `titleToSlug`. Both are slug manipulation utilities and belong together.

## Testing

Unit tests in `frontend/src/lib/slug.test.ts` covering:
- Base slug with no existing versions
- Base slug with existing versions (sequential and gaps)
- Already-versioned input slug
- Length truncation at 40 characters
- Empty existing slugs array

## Non-goals

- No API calls. This is a pure function; the caller provides `existingSlugs`.
- No UI changes. The Iterate buttons are a separate feature.
