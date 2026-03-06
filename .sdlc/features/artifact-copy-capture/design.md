# Design: Artifact and Dialogue Copy & Screenshot

## Visual Reference

[Mockup](mockup.html) — 3-state prototype: artifact viewer, dialogue hover, feedback states.

## Component Architecture

```
frontend/src/components/shared/
  CopyButton.tsx        (existing — add optional label prop)
  CaptureButton.tsx     (new — html2canvas dynamic import)

frontend/src/components/ponder/
  WorkspacePanel.tsx    (modified — add button cluster to artifact header)
  PartnerMessage.tsx    (modified — add hover-reveal copy icon)
  SessionBlock.tsx      (modified — wrap plain text messages in group for hover copy)
```

## CopyButton Enhancement

Add an optional `label` prop. When present, renders `<span>` text next to the icon. Backward compatible — all existing icon-only usages in `CommandBlock` continue working.

```tsx
interface CopyButtonProps {
  text: string
  label?: string       // new — "MD", "IMG", etc.
  className?: string
  title?: string       // new — tooltip override
}
```

**Rendered variants:**

```
Icon-only (existing):    [📋]
With label:              [📋 MD]   [📋 IMG]
```

## CaptureButton (new component)

```tsx
interface CaptureButtonProps {
  targetRef: RefObject<HTMLElement>
  mode: 'copy' | 'download'
  label?: string
  filename?: string    // download mode only, default 'artifact.png'
  title?: string
  className?: string
}
```

**State machine:**
```
idle → (click) → capturing → (html2canvas done) → done → (2s timeout) → idle
                                                  → error (clipboard denied → auto-download → done)
```

**Implementation sketch:**
```tsx
type State = 'idle' | 'capturing' | 'done' | 'error'

async function capture() {
  setState('capturing')
  try {
    const { default: html2canvas } = await import('html2canvas')
    const canvas = await html2canvas(targetRef.current!, {
      useCORS: true,
      backgroundColor: '#0e0e12', // match dark bg
    })
    if (mode === 'copy') {
      await copyCanvasToClipboard(canvas)
    } else {
      downloadCanvas(canvas, filename)
    }
    setState('done')
  } catch {
    if (mode === 'copy') {
      downloadCanvas(canvas, filename) // fallback
    }
    setState('done') // still show success — user gets the file
  }
  setTimeout(() => setState('idle'), 2000)
}

async function copyCanvasToClipboard(canvas: HTMLCanvasElement) {
  const blob = await new Promise<Blob>((res, rej) =>
    canvas.toBlob(b => b ? res(b) : rej(), 'image/png')
  )
  await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
}

function downloadCanvas(canvas: HTMLCanvasElement, name: string) {
  const a = document.createElement('a')
  a.href = canvas.toDataURL('image/png')
  a.download = name
  a.click()
}
```

## WorkspacePanel Integration

Add a `contentRef` on the artifact content div. When `activeArtifact` is non-null, render the button cluster in the artifact header.

**Button visibility rules:**
- Markdown artifacts (`.md`, `.markdown`): show [MD] [IMG] [↓]
- Code/text artifacts (`.ts`, `.rs`, `.json`, etc.): show [MD] [IMG] [↓]
- HTML artifacts (`.html`, `.htm`): show [MD] only — iframe cannot be captured

```tsx
const isHtml = ['html', 'htm'].includes(ext)
const contentRef = useRef<HTMLDivElement>(null)

// In header:
<CopyButton text={activeArtifact.content} label="MD" title="Copy markdown" />
{!isHtml && (
  <>
    <CaptureButton targetRef={contentRef} mode="copy" label="IMG" title="Copy as image" />
    <CaptureButton targetRef={contentRef} mode="download" title="Download PNG"
      filename={`${activeArtifact.filename}.png`} />
  </>
)}
```

## Dialogue Message Copy

### PartnerMessage (partner/agent bubbles)

The message bubble div gets `group relative` classes. Copy button added as an absolute element:

```tsx
<div className="relative group bg-[...] rounded-lg px-3 py-2 ...">
  {/* message content */}
  <CopyButton
    text={content}
    className={cn(
      'absolute top-1.5 right-1.5 p-1 rounded border border-border/20 bg-background/60',
      'opacity-0 group-hover:opacity-100 transition-opacity',
      isTouchDevice && 'opacity-100', // always visible on touch
    )}
    title="Copy message"
  />
</div>
```

### Touch detection

```tsx
const isTouchDevice =
  typeof window !== 'undefined' &&
  window.matchMedia('(pointer: coarse)').matches
```

This is evaluated once at module level — no React hook needed, no re-renders.

## Button Visual Design

Consistent with existing `CopyButton` style:

```
Idle:        border-border/50 bg-muted/60 text-muted-foreground
Hover:       border-border   bg-muted    text-foreground
Capturing:   border-border/50 bg-muted/60 text-muted-foreground + spinner icon
Done:        border-green-800/50 bg-green-950/60 text-green-400 + check icon
Error:       (auto-fallback to download, show Done state)
```

Labels ("MD", "IMG") are `text-[10px] font-medium` — same weight as existing label text in CommandBlock.

Download button ([↓]) is icon-only with `title="Download PNG"`.
