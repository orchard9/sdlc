# Design: Weekly Rotating Quote System for Clean Git State

## Overview

A pure frontend feature consisting of two modules: a static quote corpus and a presentation component that renders inside the git status chip when severity is green.

## Module Structure

### `frontend/src/lib/quotes.ts`

Static data module exporting the quote corpus and a selector function.

```typescript
export interface Quote {
  text: string;
  author: string;
}

export const QUOTES: Quote[] = [
  // 12+ curated developer/craft quotes
];

/** Returns the quote for the current week. Deterministic: same week = same quote. */
export function getWeeklyQuote(quotes: Quote[] = QUOTES): Quote {
  const weekIndex = Math.floor(Date.now() / (7 * 24 * 60 * 60 * 1000));
  return quotes[weekIndex % quotes.length];
}
```

### `frontend/src/components/GitGreenQuote.tsx`

Presentational component that receives a `Quote` and renders it as a styled blockquote.

```typescript
interface GitGreenQuoteProps {
  quote?: Quote;  // defaults to getWeeklyQuote()
}
```

Rendering rules:
- Italic quote text in a muted/green-tinted blockquote style.
- Author attribution below the text, right-aligned, prefixed with an em-dash.
- Compact layout — fits within the existing status chip vertical space.

### Integration Point

The `GitStatusChip` component (from `git-status-chip` feature) conditionally renders `<GitGreenQuote />` when the severity from the git status API is `green`. When severity is not green, the normal status summary is shown instead.

```
if severity === "green"
  render <GitGreenQuote />
else
  render <StatusSummary ... />
```

## Data Flow

```
getWeeklyQuote() ─── Quote ──► GitGreenQuote ──► rendered in GitStatusChip
                                                   (only when severity=green)
```

No API calls. No state management. The quote is computed on render from the current timestamp.

## Styling

- Uses existing design tokens (CSS variables) from the project theme.
- Quote text: `var(--color-muted)` or a green-tinted variant.
- Font: italic, slightly smaller than body text.
- Author: regular weight, smaller size, right-aligned.

## Testing Strategy

- Unit test `getWeeklyQuote`: verify determinism (same timestamp = same quote), verify rotation (different weeks = different quotes over the cycle).
- Unit test `GitGreenQuote` component: renders quote text and author.
- No integration test needed — the integration is a single conditional render in the chip.

[Mockup](mockup.html)
