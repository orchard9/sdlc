# Review: Fix AssistantTextBlock Missing Markdown Rendering

## Change Summary

Replaced the plain `<p>` element in `AssistantTextBlock.tsx` with the existing `CompactMarkdown` component. One file changed, one import added.

## Code Review

### `frontend/src/components/runs/AssistantTextBlock.tsx`

| # | Finding | Severity | Action |
|---|---------|----------|--------|
| 1 | Import added correctly, path matches existing convention (`@/components/shared/CompactMarkdown`) | OK | None |
| 2 | `className="text-foreground/90"` passes through correctly via `CompactMarkdown`'s `cn('compact-md', className)` | OK | None |
| 3 | Empty text guard (`if (!event.text.trim()) return null`) preserved — `CompactMarkdown` has its own guard but the outer one avoids rendering the `<div className="py-1">` wrapper unnecessarily | OK | None |
| 4 | TypeScript compiles cleanly (`npx tsc --noEmit` passes) | OK | None |

## Verdict

No issues found. The change is minimal, correct, and uses the existing component as intended.
