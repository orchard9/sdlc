# Code Review: File Browser Component

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/hooks/useGitFiles.ts` | New — custom hook for fetching git file list |
| `frontend/src/components/git/StatusBadge.tsx` | New — reusable status badge component |
| `frontend/src/components/git/GitFileBrowser.tsx` | New — main file browser with flat/tree views, filters, keyboard nav |
| `frontend/src/pages/GitPage.tsx` | Modified — integrated GitFileBrowser into the Files tab |

## Review Findings

### Correctness

1. **useGitFiles hook** follows the established pattern from `useGitStatus` precisely — polling, visibility-pause, focus-refetch. The API response shape (`{ files: [...] }`) is correctly handled with a null-coalescing fallback (`data.files ?? []`).

2. **Filter logic** correctly implements all four filters. The `modified` filter checks `status === 'M'` which matches the git porcelain output. The `staged` filter checks the boolean `staged` field. The `untracked` filter checks `status === '??'`.

3. **Tree building** correctly splits paths, creates intermediate directories, sorts directories before files, and aggregates counts. The `flattenTree` function correctly respects the `expandedDirs` set.

4. **Keyboard navigation** properly prevents default on handled keys, clamps cursor to valid bounds, and ignores events when the target is an input/textarea/select.

5. **Integration** correctly wires `onFileSelect` through to `navigate()` so file selection updates the URL and detail pane.

### Code Quality

6. **Component decomposition** is clean — `StatusBadge`, `FlatFileRow`, `DirectoryRow`, `TreeFileRow`, `LoadingState`, `ErrorState`, `EmptyState` are all extracted as focused sub-components.

7. **TypeScript types** are well-defined — `GitFile`, `TreeNodeData`, `FilterType`, `ViewMode` are all explicit interfaces/types.

8. **localStorage persistence** for view mode uses try/catch for SSR safety and environments where storage is unavailable.

### Minor Items

9. **No `unwrap()` equivalent in TS** — all optional accesses use `?.` or `??` appropriately. No runtime crashes from null access.

10. **Scroll-into-view** uses `scrollIntoView({ block: 'nearest' })` which is the correct option to avoid jarring jumps.

## Verdict

All findings are positive. The implementation follows the spec and design faithfully, uses established patterns from the codebase, type-checks cleanly, and integrates properly with the existing GitPage shell. No issues found.
