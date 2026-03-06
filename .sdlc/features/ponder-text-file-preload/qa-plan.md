# QA Plan: Text File Preload in NewIdeaModal

## Scope

All tests target `frontend/src/components/ponder/NewIdeaModal.tsx`. No backend changes exist. QA is performed by code inspection and browser interaction via the running `sdlc-server` UI.

## Test Cases

### TC-01 — Files section renders

**Given** the New Idea modal is open
**Then** a "Files (optional)" section is visible between References and the footer
**And** the drop zone shows the upload icon, "Drop files here or click to browse" text, and extension hint

### TC-02 — Click to browse opens file picker

**Given** the modal is open
**When** the user clicks the drop zone
**Then** the browser native file picker opens
**And** it is filtered to accepted extensions (.md, .txt, .html, .svg, etc.)

### TC-03 — File added via file picker

**Given** the user selects one or more accepted files via the file picker
**Then** each file appears as a chip below the drop zone
**And** the chip shows the correct filename and human-readable size

### TC-04 — File added via drag and drop

**Given** the user drags a file over the drop zone
**Then** the drop zone border changes to primary color and shows "Release to attach"
**When** the user drops the file
**Then** the file appears as a chip

### TC-05 — Remove file chip

**Given** one or more files are attached
**When** the user clicks the `×` on a file chip
**Then** that file is removed from the list
**And** the remaining files are unaffected

### TC-06 — Deduplication

**Given** a file named `spec.md` is already attached
**When** the user attaches `spec.md` again
**Then** only one chip for `spec.md` exists (no duplicate)

### TC-07 — Unaccepted file type ignored

**Given** the user drops or selects a `.pdf`, `.png`, or `.docx` file
**Then** no chip is added for that file
**And** accepted files in the same selection are still added

### TC-08 — Large file warning

**Given** the user attaches a file larger than 500 KB
**Then** a `⚠` warning indicator appears on the chip
**And** the file is still included (not rejected)

### TC-09 — Create Idea with files — artifacts captured

**Given** one or more files are attached
**When** the user submits the form
**Then** `capturePonderArtifact` is called once per attached file with the correct filename and file content
**And** the ponder entry is created successfully
**And** the chat session starts with a seed that includes "Preloaded files: …"

**Verification method:** Browser DevTools Network tab — observe POST requests to `/api/roadmap/<slug>/capture`

### TC-10 — Create Idea without files — no regression

**Given** no files are attached
**When** the user submits the form
**Then** no calls to `capturePonderArtifact` are made
**And** seed message equals `"<title>\n\n<brief>"` (same as pre-feature behavior when brief is present, or just title when absent)

### TC-11 — Reset on re-open

**Given** the modal was opened and files were attached, then closed
**When** the modal is opened again
**Then** the file list is empty (no stale files from the previous session)

### TC-12 — TypeScript / linter passes

Run `cd frontend && npm run build` (or `tsc --noEmit`).
**Then** no TypeScript errors related to the new code

## Pass Criteria

All 12 test cases pass. No TypeScript errors. No regressions in existing ponder creation flow (TC-10, TC-11).
