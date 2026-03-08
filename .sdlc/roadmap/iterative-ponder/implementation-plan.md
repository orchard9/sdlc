# Implementation Plan

## Files to Change

### 1. New utility: `frontend/src/lib/iterate.ts`
```typescript
/**
 * Given a base slug (possibly already versioned), compute the next
 * iteration slug by scanning existing ponder slugs.
 * 
 * Examples:
 *   nextIterationSlug('git-status-indicator', []) → 'git-status-indicator-v2'
 *   nextIterationSlug('git-status-indicator', ['git-status-indicator-v2']) → 'git-status-indicator-v3'
 *   nextIterationSlug('git-status-indicator-v2', ['git-status-indicator-v2', 'git-status-indicator-v3']) → 'git-status-indicator-v4'
 */
export function nextIterationSlug(slug: string, existingSlugs: string[]): string {
  // Strip trailing -vN to find base
  const base = slug.replace(/-v\d+$/, '')
  const pattern = new RegExp(`^${base}-v(\d+)$`)
  
  let maxVersion = 1 // original is implicitly v1
  for (const s of existingSlugs) {
    const match = s.match(pattern)
    if (match) {
      maxVersion = Math.max(maxVersion, parseInt(match[1], 10))
    }
  }
  
  const nextSlug = `${base}-v${maxVersion + 1}`
  return nextSlug.slice(0, 40) // respect slug length limit
}
```

### 2. `frontend/src/components/milestones/ReleasedPanel.tsx`
- Import `NewIdeaModal` and `nextIterationSlug`
- Add `useState` for iterate modal open state
- Fetch ponder slugs (from `api.getPonderEntries()` or pass down)
- Add "Iterate" button in the actions row (next to Re-run UAT)
- Render `NewIdeaModal` with initial props:
  - `initialTitle`: milestone title
  - `initialSlug`: `nextIterationSlug(milestoneSlug, ponderSlugs)`
  - `initialBrief`: template with milestone title + vision

### 3. `frontend/src/pages/FeatureDetail.tsx`
- Same pattern: add Iterate button in the released panel (lines 221-243)
- Use feature title, slug, and description for the brief

### 4. `frontend/src/components/ponder/NewIdeaModal.tsx`
- No changes needed! Already accepts initial* props.

## Button Style
Match the existing button style in ReleasedPanel — small, bordered, green accent:
```tsx
<button
  onClick={() => setIterateOpen(true)}
  className="shrink-0 inline-flex items-center gap-1 px-2.5 py-1 rounded border border-primary/30 bg-primary/20 text-primary text-[11px] hover:bg-primary/30 transition-colors"
>
  <RefreshCw className="w-3 h-3" />
  Iterate
</button>
```

## Brief Template (Milestone)
```
Iteration of milestone: {title} ({slug})

Original vision:
{vision || 'No vision recorded.'}

What worked well, what to improve, and what to explore next:
```

## Brief Template (Feature)
```
Iteration of feature: {title} ({slug})

{description || ''}

What worked well, what to improve, and what to explore next:
```