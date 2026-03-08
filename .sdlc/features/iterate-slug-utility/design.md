# Design: nextIterationSlug utility

## Module Location

Add to `frontend/src/lib/slug.ts` alongside the existing `titleToSlug` function.

## Algorithm

```
function nextIterationSlug(baseSlug: string, existingSlugs: string[]): string
  1. base = baseSlug.replace(/-v\d+$/, '')    // strip trailing -vN
  2. pattern = RegExp(`^${escapeRegex(base)}-v(\d+)$`)
  3. maxVersion = 1                            // unversioned = v1
  4. for each slug in existingSlugs:
       if match = slug.match(pattern):
         maxVersion = max(maxVersion, parseInt(match[1]))
  5. return `${base}-v${maxVersion + 1}`.slice(0, 40)
```

## Design Decisions

- **Pure function**: No side effects, no API calls. Caller provides the list of existing slugs.
- **Colocated**: Lives in `slug.ts` with `titleToSlug` since both are slug manipulation utilities.
- **Regex escape**: The base slug is escaped before building the match pattern to avoid regex injection from slugs containing special characters (unlikely but defensive).
- **40-char cap**: Matches the slug length limit used elsewhere in the system.
- **Gap tolerance**: If versions v2, v3, v5 exist (v4 missing), the function returns v6. It finds the max, not the next gap.

## Test File

`frontend/src/lib/slug.test.ts` — colocated with the source.
