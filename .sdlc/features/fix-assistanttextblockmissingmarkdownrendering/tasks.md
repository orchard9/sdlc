# Tasks: Fix AssistantTextBlock Missing Markdown Rendering

## Tasks

- [ ] Replace `<p>` with `<CompactMarkdown>` in `AssistantTextBlock.tsx` — import `CompactMarkdown` from `@/components/shared/CompactMarkdown`, replace the `<p>` element with `<CompactMarkdown content={event.text} className="text-foreground/90" />`
