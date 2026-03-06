# Spec: CompactMarkdown Component

## Problem

Activity feed cards (`AssistantTextBlock`, `RunResultCard`, `RunInitCard`, `ToolCallCard`) render text content as plain text using `whitespace-pre-wrap`. Agent output frequently contains markdown formatting — bold, inline code, lists, links, code fences — that appears as raw syntax characters rather than rendered markup.

The existing `MarkdownContent` component is too heavyweight for activity feed use: it includes a table of contents sidebar, raw/rendered toggle, mermaid diagram support, syntax-highlighted code blocks with `react-syntax-highlighter`, and heading anchors. Using it directly in the activity feed would add visual clutter, excessive DOM weight, and inappropriate chrome for inline card content.

## Solution

Create a `CompactMarkdown` component — a lightweight, inline-optimized markdown renderer designed for tight spaces like activity feed cards. It renders a minimal subset of markdown (bold, italic, inline code, links, lists, code fences) without the TOC, raw toggle, mermaid, or syntax highlighting of `MarkdownContent`.

Replace the plain-text `<p>` elements in all four activity feed card components with `CompactMarkdown`.

## Scope

### In scope

- New `CompactMarkdown` component at `frontend/src/components/shared/CompactMarkdown.tsx`
- Uses `react-markdown` and `remark-gfm` (already dependencies)
- Renders: bold, italic, inline code, links, unordered/ordered lists, code fences (monospace, no syntax highlighting), blockquotes, strikethrough, paragraphs
- Does NOT include: TOC, raw toggle, mermaid, `react-syntax-highlighter`, heading anchors, file-path IDE links
- Typography scaled for `text-xs` / `text-[10px]` contexts — compact line height, minimal margins
- Integrate into: `AssistantTextBlock`, `RunResultCard` (result text), `RunInitCard` (prompt), `ToolCallCard` (summary)

### Out of scope

- Changing `MarkdownContent` — it continues to serve artifact viewers and full-page contexts
- Adding markdown to `AgentLog` or `AgentEventLine` (SSE live stream — stays as plain text for performance)
- Image rendering in compact contexts

## Acceptance criteria

1. `CompactMarkdown` renders bold, italic, inline code, links, lists, code fences, blockquotes, and strikethrough correctly
2. All four activity feed card components use `CompactMarkdown` instead of plain text for their content areas
3. Code fences render as monospace pre blocks without syntax highlighting
4. The component adds no new npm dependencies beyond what is already installed
5. Visual density remains comparable to the current plain-text rendering — no large margins or oversized headings
6. `MarkdownContent` is not modified
