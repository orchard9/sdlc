# Tasks: Text File Preload in NewIdeaModal

## T1 — Add file attachment state and helpers to NewIdeaModal

Add `attachedFiles` and `isDragOver` state, the `ACCEPTED_EXTS` set, `isAccepted(file)` helper, `handleFilesAdded(files)` with deduplication by name, and `handleRemoveFile(index)` to `NewIdeaModal.tsx`. Reset both state fields when the modal opens.

**File:** `frontend/src/components/ponder/NewIdeaModal.tsx`

## T2 — Add file input + drop zone UI

Add the "Files (optional)" section to the form between the References section and the error block. Include:
- A hidden `<input type="file" multiple accept="…" ref={fileInputRef} />` with all accepted extensions
- A visible drop zone `<div>` that calls `fileInputRef.current?.click()` on click, handles `onDragOver`, `onDragLeave`, and `onDrop`
- Conditional class for `isDragOver` (primary border + tint)
- Import `UploadCloud` from `lucide-react`

**File:** `frontend/src/components/ponder/NewIdeaModal.tsx`

## T3 — Render attached file chips

Below the drop zone, render a list of file chips for each item in `attachedFiles`. Each chip shows the `FileText` icon, filename, human-readable size (reuse or adapt `formatBytes` pattern), a `⚠` warning if size > 500 KB, and a remove button. Import `FileText` from `lucide-react`.

**File:** `frontend/src/components/ponder/NewIdeaModal.tsx`

## T4 — Capture files and update seed message in handleSubmit

In `handleSubmit`, after `createPonderEntry` succeeds and before `startPonderChat`:
1. For each file in `attachedFiles`: read content with `await file.text()` then call `api.capturePonderArtifact(slug, { filename: file.name, content })`
2. Build seed message as `[title, brief, fileNames ? \`Preloaded files: ${fileNames}\` : ''].filter(Boolean).join('\n\n')`
3. Pass the updated seed to `api.startPonderChat`

**File:** `frontend/src/components/ponder/NewIdeaModal.tsx`

## T5 — Verify zero-regression when no files are attached

Confirm `handleSubmit` behaves identically to the pre-change code when `attachedFiles` is empty: no extra API calls, seed message unchanged (title + brief only).

**Verification:** Code review / trace through `handleSubmit` logic.
