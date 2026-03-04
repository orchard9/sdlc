# Implementation Design

## Phase 1 — Text File Preload (NewIdeaModal only)

### Scope
Support attaching text-based files at ponder creation time: .md, .html, .svg, .txt, .ts, .tsx, .rs, .json, .yaml, .toml.

### Changes Required

**frontend/src/components/ponder/NewIdeaModal.tsx**
- Add `preloadedFiles: Array<{ filename: string, content: string }>` state
- Add a `<PreloadAttachments>` section below the Description field
- File input: `<input type="file" multiple accept="...text extensions">`
- On file pick: read as text with FileReader, add to preloadedFiles state
- Display chips: filename pill + × remove button, with file size hint
- Binary validation: if file extension is .png/.jpg/.jpeg/.gif/.webp/.pdf, show warning toast (don't add)
- Create flow: after `createPonderEntry`, before `startPonderChat`, run `capturePonderArtifact` for each file sequentially
- Seed message: append `\n\nPreloaded artifacts: file1.md, diagram.svg` if files exist

**frontend/src/api/client.ts**
- No changes needed

**Backend**
- No changes needed

### UX Details
- Drop zone: the attach area accepts file drops (dragover + drop events)
- Click-to-browse: clicking the zone opens file picker
- Visual: dashed border area with upload icon, "+Attach files" label
- Chip style: muted bg, filename truncated, size badge (KB), × button
- Error state: binary file attempted → inline error ".png files are not supported yet", file not added
- If upload of a specific file fails (after creation): warn but proceed; note partial upload in seed

### Sequence on Submit
1. api.createPonderEntry(...)
2. If brief → captured automatically by server
3. For each preloadedFile: await api.capturePonderArtifact(slug, file)
4. api.startPonderChat(slug, seedWithFileList)
5. onCreated(slug)

## Phase 2 — Binary Image Support (future feature)

### New Backend Endpoints
- `POST /api/roadmap/:slug/upload` — multipart/form-data, stores raw bytes in ponder dir
- `GET /api/roadmap/:slug/files/:filename` — serves raw file with correct Content-Type

### Frontend Changes
- `api.uploadPonderFile(slug, file: File)` — FormData post
- `ArtifactContent` — detect image extensions, render `<img src="/api/roadmap/:slug/files/:filename">`
- `WorkspacePanel` — show image thumbnail (32px square) in artifact row
- `NewIdeaModal` — extend accept list to include images, route to uploadPonderFile instead of capturePonderArtifact
- `WorkspacePanel` — optional "Attach" button for uploads on existing ponders

### Storage
- Raw binary written to `.sdlc/roadmap/<slug>/<filename>`
- Artifact listing includes binary files (size_bytes from fs metadata, content: null)
- Frontend detects null content + image extension → use file serve URL

## Commit Signal
Phase 1 ready to spec when: scope confirmed, no open questions about text-file attachment in NewIdeaModal.