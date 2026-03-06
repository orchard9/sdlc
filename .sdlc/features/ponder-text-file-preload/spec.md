# Spec: Text File Preload in NewIdeaModal

## Summary

Allow users to attach text files (`.md`, `.html`, `.svg`, and common code file extensions) when creating a new ponder idea via the `NewIdeaModal`. Attached files are captured as scrapbook artifacts before the ponder chat session starts, so the agent's first message is seeded with their content.

## Problem

Currently, the `NewIdeaModal` only accepts a title, slug, optional description, and URL references. Users frequently have existing documents, specs, or code snippets they want to use as context for a new ponder session. There is no way to include file content at creation time — users must manually capture files after the ponder is created and before a meaningful agent session can begin.

## Goals

- Let users attach one or more text files at ponder creation time
- Accepted file types: `.md`, `.txt`, `.html`, `.svg`, `.js`, `.ts`, `.tsx`, `.jsx`, `.rs`, `.py`, `.go`, `.json`, `.yaml`, `.yml`, `.toml`, `.css`, `.sh`
- Each attached file is captured as a scrapbook artifact (`capturePonderArtifact`) under the ponder slug using its original filename
- File content is readable as plain text (binary files are excluded by type)
- The seed message passed to `startPonderChat` should include a summary of attached file names so the agent is aware of the preloaded context

## Non-Goals

- Binary file support (images, PDFs, etc.) — out of scope for this feature
- Editing file content in the modal — files are read-only at attach time
- Backend changes — all logic lives in the frontend; the existing `capturePonderArtifact` API endpoint is sufficient

## User Flow

1. User opens "New Idea" modal
2. User fills in Title, Slug, Description (as today)
3. New "Files" section below References: click to browse or drag-and-drop accepted file types
4. Selected files show in a list with name + size; user can remove individual files
5. User clicks "Create Idea"
6. Modal calls `createPonderEntry` as today
7. For each attached file: reads content and calls `capturePonderArtifact(slug, { filename, content })`
8. If URL references exist, captures `references.md` as today
9. Calls `startPonderChat` with a seed message that mentions attached file names (e.g., "Attached files: spec.md, notes.md")
10. Modal closes and navigates to the new ponder entry

## Acceptance Criteria

1. A "Files" section appears in the NewIdeaModal between References and the footer
2. Clicking the file input (or drop zone) opens a file browser filtered to accepted extensions
3. Multiple files can be selected and/or dropped
4. Each selected file shows as a chip/row with filename and human-readable size
5. Individual files can be removed before submission
6. On submit, each file's text content is captured as a ponder scrapbook artifact
7. The ponder chat seed message includes the names of attached files
8. Non-accepted file types are silently ignored (or show an inline error)
9. Large files (>500 KB each) show a warning but are still allowed
10. The feature works correctly when zero files are attached (no regression)

## Technical Notes

- Use `FileReader.readAsText()` or the `text()` method on `File` objects to read content
- The existing `capturePonderArtifact` API endpoint accepts `{ filename: string; content: string }` — no server changes needed
- Extend the `handleSubmit` function in `NewIdeaModal.tsx` to iterate `attachedFiles` state and call `capturePonderArtifact` for each
- Seed message: `"{title}\n\n{brief}\n\nPreloaded files: {fileNames}"`
- Use a file input with `accept` attribute listing all supported extensions; also handle drag-and-drop via `onDrop` / `onDragOver`
- File reading is done client-side — no server changes to `sdlc-server`
