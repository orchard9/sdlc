# Design: Remove Artifact Height Cap

## Overview

This is a single-line CSS change. No new components, no new state, no backend changes. The design is straightforward: remove `max-h-96 overflow-y-auto` from the artifact content wrapper in `ArtifactViewer.tsx`.

## Current Behavior

`ArtifactViewer.tsx` line 36:
```tsx
<div className="p-4 max-h-96 overflow-y-auto">
  <MarkdownContent content={artifact.content} />
</div>
```

`max-h-96` constrains the content div to 384px. When artifact content exceeds that height, a vertical scrollbar appears inside the card. Combined with the page-level scroll, users see a "double scrollbar" — one inside the card, one for the page — which makes reading long artifacts (spec, design, tasks) difficult.

## Proposed Change

```tsx
<div className="p-4">
  <MarkdownContent content={artifact.content} />
</div>
```

Remove `max-h-96 overflow-y-auto`. The artifact card expands to its natural content height. The page scrolls as a whole — standard browser behavior.

## Layout Impact

- `ArtifactViewer` renders inside `FeatureDetail.tsx` which has `max-w-4xl mx-auto p-6` — no horizontal overflow.
- The `border border-border rounded-lg overflow-hidden` on the outer card wrapper already clips any unexpected overflow at the card edge.
- `overflow-hidden` on the outer wrapper remains; the only change is removing the inner height cap and inner scroll.

## Fullscreen Button

Unaffected. The fullscreen button is in the card header (`flex items-center justify-between` row) and has no dependency on the content div's height constraints.

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/components/features/ArtifactViewer.tsx` | Remove `max-h-96 overflow-y-auto` from content div className (line 36) |

## What is NOT in Scope

- TOC navigation (`artifact-fullscreen-toc`)
- File link detection (`artifact-file-links`)
- Card teaser (`artifact-tldr-teaser`)
- Any backend changes
- Any new components
