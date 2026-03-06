# Design: Text File Preload in NewIdeaModal

## Overview

This is a pure frontend change to `frontend/src/components/ponder/NewIdeaModal.tsx`. No Rust, server, or API changes are required — the existing `capturePonderArtifact` endpoint already accepts arbitrary filename + content pairs.

## UI Design

See [Mockup](mockup.html) for the interactive screen states.

### Layout

The modal gains a new **"Files"** section between the References section and the form footer. The section follows the same visual pattern as References (label + optional hint + content area).

```
┌─────────────────────────────────────────────────────┐
│  New Idea                                      [X]  │
├─────────────────────────────────────────────────────┤
│  Title                                              │
│  ┌─────────────────────────────────────────────┐   │
│  │ What are you thinking about?                │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  Slug                                               │
│  ┌─────────────────────────────────────────────┐   │
│  │ slug                                        │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  Description  (optional)                            │
│  ┌─────────────────────────────────────────────┐   │
│  │                                             │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  References  (optional)                             │
│  ┌─ 🔗 https://...  ─────────────────────────[X]┐  │
│  └──────────────────────────────────────────────┘  │
│  + Add reference                                    │
│                                                     │
│  Files  (optional)                                  │
│  ┌─────────────────────────────────────────────┐   │
│  │  ⬆ Drop files here or click to browse      │   │
│  │  .md .txt .html .svg .js .ts .rs .py …     │   │
│  └─────────────────────────────────────────────┘   │
│  [📄 spec.md · 4.2 KB  ×]                          │
│  [📄 notes.txt · 1.1 KB ×]                         │
├─────────────────────────────────────────────────────┤
│                          [Cancel]  [Create Idea]    │
└─────────────────────────────────────────────────────┘
```

### Drop Zone States

- **Empty/idle**: dashed border, muted text "Drop files here or click to browse", accepted extensions hint
- **Drag over**: border color shifts to primary, background tint
- **Files attached**: list of file chips appears below the drop zone; zone remains interactive for adding more files

### File Chip

Each attached file shows:
- `📄` icon + filename + `·` + human-readable size
- `×` remove button on the right
- Warning icon if size > 500 KB

### Accepted Extensions

`.md .txt .html .svg .js .ts .tsx .jsx .rs .py .go .json .yaml .yml .toml .css .sh`

## Component State Changes (`NewIdeaModal.tsx`)

### New state

```ts
const [attachedFiles, setAttachedFiles] = useState<File[]>([])
const [isDragOver, setIsDragOver] = useState(false)
```

### File handling

```ts
const ACCEPTED_EXTS = new Set([
  '.md', '.txt', '.html', '.svg', '.js', '.ts', '.tsx', '.jsx',
  '.rs', '.py', '.go', '.json', '.yaml', '.yml', '.toml', '.css', '.sh'
])

function isAccepted(file: File): boolean {
  const ext = '.' + file.name.split('.').pop()?.toLowerCase()
  return ACCEPTED_EXTS.has(ext)
}

function handleFilesAdded(files: FileList | null) {
  if (!files) return
  const accepted = Array.from(files).filter(isAccepted)
  setAttachedFiles(prev => {
    // deduplicate by name
    const existing = new Set(prev.map(f => f.name))
    return [...prev, ...accepted.filter(f => !existing.has(f.name))]
  })
}

function handleRemoveFile(index: number) {
  setAttachedFiles(prev => prev.filter((_, i) => i !== index))
}
```

### Submit changes

```ts
// After createPonderEntry succeeds and before startPonderChat:
for (const file of attachedFiles) {
  const content = await file.text()
  await api.capturePonderArtifact(slug.trim(), { filename: file.name, content })
}

// Modify seed message to include file names:
const fileNames = attachedFiles.map(f => f.name).join(', ')
const seed = [
  title.trim(),
  brief.trim(),
  fileNames ? `Preloaded files: ${fileNames}` : ''
].filter(Boolean).join('\n\n')
api.startPonderChat(slug.trim(), seed).catch(() => {})
```

### Reset on open

```ts
setAttachedFiles([])
setIsDragOver(false)
```

## Drag and Drop

```tsx
<div
  onDragOver={e => { e.preventDefault(); setIsDragOver(true) }}
  onDragLeave={() => setIsDragOver(false)}
  onDrop={e => {
    e.preventDefault()
    setIsDragOver(false)
    handleFilesAdded(e.dataTransfer.files)
  }}
  onClick={() => fileInputRef.current?.click()}
  className={cn(
    "cursor-pointer border border-dashed rounded-lg p-4 text-center transition-colors",
    isDragOver
      ? "border-primary/60 bg-primary/5"
      : "border-border hover:border-primary/40"
  )}
>
  <UploadCloud className="w-4 h-4 mx-auto mb-1 text-muted-foreground/50" />
  <p className="text-xs text-muted-foreground">Drop files here or click to browse</p>
  <p className="text-[10px] text-muted-foreground/50 mt-0.5">.md .txt .html .svg .js .ts .rs .py …</p>
</div>
<input
  ref={fileInputRef}
  type="file"
  multiple
  accept=".md,.txt,.html,.svg,.js,.ts,.tsx,.jsx,.rs,.py,.go,.json,.yaml,.yml,.toml,.css,.sh"
  className="hidden"
  onChange={e => handleFilesAdded(e.target.files)}
/>
```

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/components/ponder/NewIdeaModal.tsx` | Add file attach UI and submission logic |

No other files need to change.
