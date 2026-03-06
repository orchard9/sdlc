# Design: Fix AssistantTextBlock Missing Markdown Rendering

## Change

Replace the plain `<p>` element in `AssistantTextBlock.tsx` with the existing `CompactMarkdown` component.

## Before

```tsx
<p className="text-xs text-foreground/90 whitespace-pre-wrap leading-relaxed">
  {event.text}
</p>
```

## After

```tsx
<CompactMarkdown content={event.text} className="text-foreground/90" />
```

## Rationale

- `CompactMarkdown` already renders markdown with `text-xs` sizing, `leading-relaxed`, and handles code fences, inline code, bold, lists, tables, and GFM.
- The `className` prop passes through `text-foreground/90` for color consistency.
- No wrapper `<div className="py-1">` change needed — it stays as the outer container.

## Files Modified

| File | Change |
|------|--------|
| `frontend/src/components/runs/AssistantTextBlock.tsx` | Import `CompactMarkdown`, replace `<p>` with `<CompactMarkdown>` |

## No New Components or Dependencies

This is a one-import, one-line swap.
