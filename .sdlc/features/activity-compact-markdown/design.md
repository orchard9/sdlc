# Design: CompactMarkdown Component

## Component API

```tsx
interface CompactMarkdownProps {
  content: string
  className?: string
}

export function CompactMarkdown({ content, className }: CompactMarkdownProps)
```

## Architecture

### Component location

`frontend/src/components/shared/CompactMarkdown.tsx`

### Dependencies (all existing)

- `react-markdown` — markdown parsing and rendering
- `remark-gfm` — GFM extensions (strikethrough, tables, task lists)
- `@/lib/utils` — `cn()` for class merging

### Rendering strategy

Uses `ReactMarkdown` with a stripped-down `components` override map. All elements use `text-xs` scale with compact spacing:

| Element | Rendering |
|---|---|
| `p` | `text-xs leading-relaxed mb-1 last:mb-0` |
| `strong` | `font-semibold text-foreground` |
| `em` | `italic text-muted-foreground` |
| `del` | `line-through text-muted-foreground` |
| `code` (inline) | `text-[10px] font-mono bg-muted/60 border border-border/50 px-1 py-0.5 rounded` |
| `code` (block) | `text-[10px] font-mono bg-muted/40 border border-border/40 rounded p-2 whitespace-pre-wrap overflow-x-auto` — no syntax highlighting |
| `pre` | Wrapper div with `mb-1` |
| `a` | `text-primary underline underline-offset-2 hover:opacity-80` |
| `ul` | `list-disc pl-4 mb-1 space-y-0.5 text-xs` |
| `ol` | `list-decimal pl-4 mb-1 space-y-0.5 text-xs` |
| `li` | `text-xs leading-relaxed` |
| `blockquote` | `border-l-2 border-border pl-2 my-1 text-muted-foreground italic text-xs` |
| `h1-h3` | All render as `text-xs font-semibold` — no size differentiation in compact mode |
| `hr` | `border-border my-2` |
| `table` | Not rendered (returns children as text) — tables are too wide for card contexts |

### Integration points

1. **AssistantTextBlock** — replace `<p className="text-xs ...whitespace-pre-wrap">{event.text}</p>` with `<CompactMarkdown content={event.text} className="text-foreground/90" />`

2. **RunResultCard** — replace the result text `<p>` with `<CompactMarkdown content={event.text} className="text-muted-foreground/80" />`

3. **RunInitCard** — replace the prompt `<p>` with `<CompactMarkdown content={prompt} className="text-muted-foreground/80" />`

4. **ToolCallCard** — replace the summary `<p>` with `<CompactMarkdown content={event.summary} className="text-muted-foreground/70" />`

### What is NOT included

- No TOC, no raw toggle, no mermaid, no `react-syntax-highlighter`
- No heading anchors or slug generation
- No file-path IDE link detection
- No `useProjectSettings` dependency

## File changes

| File | Change |
|---|---|
| `frontend/src/components/shared/CompactMarkdown.tsx` | New file — component |
| `frontend/src/components/runs/AssistantTextBlock.tsx` | Import + use CompactMarkdown |
| `frontend/src/components/runs/RunResultCard.tsx` | Import + use CompactMarkdown |
| `frontend/src/components/runs/RunInitCard.tsx` | Import + use CompactMarkdown |
| `frontend/src/components/runs/ToolCallCard.tsx` | Import + use CompactMarkdown |
