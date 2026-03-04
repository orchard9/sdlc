# Context Map

## Current State

**NewIdeaModal** supports:
- Title (text)
- Slug (auto-derived)  
- Description / Brief (textarea → saved as `brief.md`)
- References (URL links → saved as `references.md`)

No file attachment support.

**`POST /api/roadmap/:slug/capture`** — accepts `{ filename: string, content: string }`. Text-only. No binary.

**ArtifactContent** renders: .md (markdown), .html (iframe sandbox), other extensions as code block. No image rendering.

**Agent seed**: `title + brief` text string. No artifact context.

## Gap

Users cannot preload workspace artifacts at idea-creation time:
- No file picker in NewIdeaModal
- No binary upload endpoint
- Images not renderable in WorkspacePanel
- Agent seeded without knowledge of preloaded material

## Artifact Types Requested

| Type | Examples | Backend | Frontend |
|------|---------|---------|---------|
| Markdown | design docs, notes, specs | ✅ text capture | ✅ rendered |
| HTML | prototypes, mockups | ✅ text capture | ✅ iframe |
| SVG | diagrams, icons | ✅ text capture | ⚠️ code block |
| Images | screenshots, mockups | ❌ no binary | ❌ no img tag |
| Code | .rs, .ts, .json | ✅ text capture | ✅ code block |