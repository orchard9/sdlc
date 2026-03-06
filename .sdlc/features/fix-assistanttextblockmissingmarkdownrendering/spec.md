# Spec: Fix AssistantTextBlock Missing Markdown Rendering

## Problem

`AssistantTextBlock.tsx` renders agent output as plain text using a `<p>` tag with `whitespace-pre-wrap`. Markdown syntax (code blocks, bold, lists, inline code) appears as raw text instead of formatted HTML.

## Root Cause

The component uses `<p className="text-xs text-foreground/90 whitespace-pre-wrap leading-relaxed">{event.text}</p>` instead of the `CompactMarkdown` component that already exists at `frontend/src/components/shared/CompactMarkdown.tsx`.

## Fix

Replace the `<p>` element in `AssistantTextBlock.tsx` with `<CompactMarkdown content={event.text} className="text-foreground/90" />`.

## Scope

- **One file changed**: `frontend/src/components/runs/AssistantTextBlock.tsx`
- **No new dependencies** — `CompactMarkdown` already exists and handles code fences, inline code, bold, lists, tables, and GFM via `react-markdown` + `remark-gfm`.

## Acceptance Criteria

1. Agent text blocks in the activity feed render markdown (code fences, inline code, bold, lists) as formatted HTML.
2. Plain text without markdown renders identically to before (no visual regression).
3. Empty text blocks still return `null` (existing guard preserved).
