# Code Review: ponder-text-file-preload

## Summary

Single file changed: `frontend/src/components/ponder/NewIdeaModal.tsx`. No backend changes. Pure frontend addition.

## Diff Review

### Imports and helpers (lines 1-24)

- Correct icons imported: `UploadCloud`, `FileText` from `lucide-react`
- `cn` from `@/lib/utils` — consistent with codebase pattern
- `formatBytes` is a local helper — appropriate since this is the only component that needs it at this scope; not duplicated elsewhere
- `ACCEPTED_EXTS` as a module-level `Set` — correct; avoids re-creation on every render
- `isAccepted` handles files without extensions correctly (returns false when `parts.length < 2`)

### State additions (lines 52-55)

- `attachedFiles: File[]` — correct type, initialized to `[]`
- `isDragOver: boolean` — correct type
- `fileInputRef` — correct use of `useRef<HTMLInputElement>(null)`

### Reset on open (lines 69-71)

- `setAttachedFiles([])` and `setIsDragOver(false)` are included in the `useEffect` reset block — satisfies TC-11 (no stale files on re-open)

### File helpers (lines 114-126)

- `handleFilesAdded`: filters to accepted extensions, deduplicates by name using a `Set`. Both are correct.
- `handleRemoveFile`: index-based removal — correct, file list has no stable key besides index in this context
- Input `onChange` resets `e.target.value = ''` — this allows the same file to be re-selected after removal, which is good practice

### handleSubmit (lines 151-164)

- `for...of` loop with `await` is correct — sequential capture is intentional (ordered artifacts)
- `file.text()` is the modern Web API for reading File as string — correct
- Seed message uses `[...].filter(Boolean).join('\n\n')` — cleanly handles empty brief and no-files cases, preserving backward-compatible behavior when neither brief nor files are present
- **No regression**: when `attachedFiles` is empty, the `for` loop body never executes, and `fileNames` is `''`, so it is filtered out of the seed array, giving the same output as before

### JSX (lines 293-374)

- `role="button"` + `tabIndex={0}` + `onKeyDown` on the drop zone — correct accessibility implementation
- `accept` attribute on the hidden input matches `ACCEPTED_EXTS` — consistent
- `isDragOver` conditional class uses `cn()` — correct
- File chips: `truncate` class on filename prevents overflow; size displayed; large-file `⚠` warning with tooltip
- `aria-label={`Remove ${file.name}`}` on each remove button — accessible

## Findings

### Finding 1 — TypeScript: clean

`npx tsc --noEmit` exits with zero errors. All types are correct.

### Finding 2 — No server changes needed

`capturePonderArtifact` accepts `{ filename: string; content: string }` — this API already exists and handles the new use case. Confirmed no backend changes needed.

### Finding 3 — Drag-leave edge case (fixed)

`onDragLeave` fires when hovering over child elements inside the drop zone. The fix is `onDragLeave={e => { if (!e.currentTarget.contains(e.relatedTarget as Node)) setIsDragOver(false) }}` which checks that the pointer actually left the zone rather than entering a child element. Applied in the same review pass. TypeScript verified clean afterward.

**Action:** Fixed.

### Finding 4 — `formatBytes` duplication (fixed)

`WorkspacePanel.tsx` already defined a `formatBytes` with identical logic. Extracted to `@/lib/utils` and both components updated to import from there. Local definitions removed. TypeScript verified clean.

## Verdict

Implementation is correct, complete, and consistent with the codebase patterns. All acceptance criteria from the spec are met. No blocking issues.
