# Tasks: Artifact and Dialogue Copy & Screenshot

## T1 — Add label prop to CopyButton

**File:** `frontend/src/components/shared/CopyButton.tsx`

Add an optional `label?: string` prop. When provided, render it as `<span className="text-[10px] font-medium">{label}</span>` next to the icon. When absent, icon-only (all existing usages in `CommandBlock` are unaffected). Also add `title?: string` prop to override the default `"Copy command"` tooltip.

## T2 — Create CaptureButton component

**File:** `frontend/src/components/shared/CaptureButton.tsx` (new)

Props: `targetRef`, `mode: 'copy' | 'download'`, `label?`, `filename?`, `title?`, `className?`

States: `idle | capturing | done`

- `idle` → (click) → `capturing`: show spinner + "Capturing…" label
- `capturing` → (html2canvas done) → `done`: show check + "Copied" or "Saved"
- `done` → (2s timeout) → `idle`

Copy mode: `html2canvas(el)` → `ClipboardItem({ 'image/png': blob })`. On clipboard failure, fall back to download (show "Saved" state, not error).
Download mode: `html2canvas(el)` → `canvas.toDataURL()` → anchor click.

Dynamic import: `const { default: html2canvas } = await import('html2canvas')`.
Background color: `'#0e0f14'` to match dark theme.

## T3 — Add copy/screenshot cluster to WorkspacePanel

**File:** `frontend/src/components/ponder/WorkspacePanel.tsx`

- Add `contentRef = useRef<HTMLDivElement>(null)` at component level
- Attach `ref={contentRef}` to the artifact content div
- When `activeArtifact` is non-null, add button cluster to artifact header row:
  - `<CopyButton text={activeArtifact.content} label="MD" title="Copy markdown" />`
  - If not HTML: `<CaptureButton targetRef={contentRef} mode="copy" label="IMG" title="Copy as image" />`
  - If not HTML: `<CaptureButton targetRef={contentRef} mode="download" title="Download PNG" filename={activeArtifact.filename + '.png'} />`
- HTML detection: `['html','htm'].includes(ext)` where `ext` is the file extension

## T4 — Add hover-reveal copy button to PartnerMessage

**File:** `frontend/src/components/ponder/PartnerMessage.tsx`

- Add `group relative` to the message bubble container div
- Add absolutely-positioned `<CopyButton>` in top-right of the bubble:
  ```tsx
  <CopyButton
    text={messageText}
    className="absolute top-1.5 right-1.5 p-1 rounded border border-border/20 bg-background/60 opacity-0 group-hover:opacity-100 transition-opacity"
    title="Copy message"
  />
  ```
- `messageText` is the plain text content of the message (not rendered HTML)

## T5 — Mobile fallback for dialogue copy

**File:** `frontend/src/components/ponder/PartnerMessage.tsx`

Add touch detection at module level:
```ts
const isTouchDevice = typeof window !== 'undefined' && window.matchMedia('(pointer: coarse)').matches
```

Conditionally remove `opacity-0 group-hover:opacity-100` classes when `isTouchDevice` is true (always visible).

Use `cn()` to compose: `cn('absolute top-1.5 right-1.5 ...', !isTouchDevice && 'opacity-0 group-hover:opacity-100 transition-opacity')`.

## T6 — Graceful HTML artifact handling

**File:** `frontend/src/components/ponder/WorkspacePanel.tsx`

Already covered by T3 (hide [IMG] and [↓] when `isHtml`). This task adds the documentation:

- Add an inline code comment above the button cluster explaining the iframe limitation:
  ```tsx
  {/* html2canvas cannot capture <iframe srcDoc> — hide image buttons for HTML artifacts */}
  ```

- Ensure the [MD] button still works for HTML artifacts (copy raw HTML source is valid and useful).
