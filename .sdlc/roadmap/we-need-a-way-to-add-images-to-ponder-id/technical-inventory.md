# Technical Inventory: Current Ponder Creation Stack

## What exists today

### Frontend: NewIdeaForm (PonderPage.tsx:95)
- Inline form in sidebar, not a modal
- Fields: title, slug (auto-derived), brief (textarea, 2 rows)
- No URL field, no file upload, no attachment capability
- On submit: calls `api.createPonderEntry` + immediately fires `api.startPonderChat`

### API: capturePonderArtifact (client.ts:120)
- POST /api/roadmap/:slug/capture
- Payload: `{ filename: string, content: string }`
- Text-only — no binary support
- Works for capturing markdown notes as scrapbook artifacts

### Server: Ponder chat (runs.rs:924)
- Uses `sdlc_query_options` — has Bash, Read, Write, Edit, Glob, Grep + sdlc MCP tools
- NO WebSearch / WebFetch (these are added only for guideline investigations at line 375)
- NO Playwright (UAT-only, lines 498-511)

### Storage: .sdlc/roadmap/<slug>/
- Flat directory: brief.md, team.yaml, scrapbook .md files, sessions/
- No binary storage, no image storage, no attachment directory
- All content is markdown/yaml text files

## What needs to change

1. Creation modal → bigger dialog, not inline form
2. Artifact attachment → storage strategy decision (text/URL vs binary)
3. URL reference field → new field in creation form
4. Ponder agent tools → WebSearch + WebFetch (easy); Playwright (medium)