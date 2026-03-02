## Summary

Remove the `max-h-96 overflow-y-auto` constraint from `ArtifactViewer.tsx` so artifact cards expand to their natural content height. This eliminates the primary cause of Xist's workaround (exporting to `.md` and reading in Agy). One CSS change; no backend changes; no new components.

## Problem

`ArtifactViewer.tsx` wraps artifact content in:
```tsx
<div className="p-4 max-h-96 overflow-y-auto">
```

`max-h-96` = 384px. At ~20px per line, this shows ~19 lines of a plan artifact that is routinely 300-600 lines. Users see a double scrollbar: the page scrolls AND the artifact card scrolls. Xist's exact words: "I can't really see that."

## Solution

In `frontend/src/components/features/ArtifactViewer.tsx`, locate the artifact content div (currently around line 36) and remove `max-h-96 overflow-y-auto`. Replace with `overflow-visible` or simply remove the height/overflow constraints.

**Before:**
```tsx
<div className="p-4 max-h-96 overflow-y-auto">
  <MarkdownContent content={artifact.content} />
</div>
```

**After:**
```tsx
<div className="p-4">
  <MarkdownContent content={artifact.content} />
</div>
```

The artifact card now expands to its natural height. The page itself scrolls — standard browser behavior, no double scrollbar.

## Acceptance Criteria

- A 400-line spec artifact renders fully without an inner scrollbar
- No layout breakage in `FeatureDetail.tsx` (`max-w-4xl mx-auto p-6` outer container is unaffected)
- The fullscreen button still appears and still works
- Visual review against at least one real plan artifact (spec + design)

## Files Changed

- `frontend/src/components/features/ArtifactViewer.tsx` — remove `max-h-96 overflow-y-auto`

## What is NOT in scope

- TOC navigation (Feature 4: `artifact-fullscreen-toc`)
- File link detection (Feature 2: `artifact-file-links`)
- Card teaser (Feature 3: `artifact-tldr-teaser`)
- Any backend changes
