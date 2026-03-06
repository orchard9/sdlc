# Exploration: Artifact Share & Capture

## Problem Statement

The sdlc UI renders artifacts and dialogues beautifully but provides no path to extract that content for sharing. Users resort to OS-level screenshot tools (memorized keybindings) and manual text selection. This friction compounds in exactly the moments that matter — reviewing a spec, sharing a ponder session, getting feedback on a design.

## Codebase Audit

**Existing infrastructure (don't redo):**
- `CopyButton.tsx` — fully functional, reusable, already deployed on `CommandBlock`
- `CommandBlock.tsx` — uses `CopyButton` for command/code blocks ✓

**Surfaces that need copy/screenshot (not yet present):**
- `WorkspacePanel` — artifact viewer (spec, design, tasks, scrapbook files) — no copy
- `SessionBlock` / `PartnerMessage` — dialogue messages — no copy

**Surfaces that are tricky:**
- `ArtifactContent` with HTML files renders in `<iframe srcDoc>` — html2canvas can't capture iframes. Acceptable tradeoff: show copy button as disabled/hidden for HTML artifacts, or only offer copy-markdown (raw HTML source).

## Decisions

⚑  Decided: **Copy markdown** → use existing `CopyButton` with `artifact.content`. Zero new deps.

⚑  Decided: **Copy as image** → html2canvas (dynamic import) → `ClipboardItem({ 'image/png': blob })`. Primary action for screenshots.

⚑  Decided: **Download PNG** → same html2canvas capture → `canvas.toDataURL()` → anchor click. Secondary fallback.

⚑  Decided: **Artifact buttons are always visible** (not hover-reveal). Users are in an intentional "view this artifact" mode. No clutter argument.

⚑  Decided: **Dialogue message copy is hover-reveal**. Messages are high-density; hover keeps visual noise low on desktop. On mobile: long-press or persistent icon if hover isn't available.

⚑  Decided: **Bundle impact mitigation** → dynamic import html2canvas (`import('html2canvas').then(...)`) so it doesn't inflate initial bundle.

⚑  Decided: **Error fallback** → if clipboard write fails (permission denied), fall back to download. Never silently fail.

⚑  Decided: **No PDF for now** — over-engineering.

?  Open: Mobile UX for dialogue hover — hover-reveal doesn't work on touch. Either always-visible icon or long-press context menu needed.

?  Open: HTML artifact capture — copy markdown (raw HTML source) still useful? Or hide both buttons for `<iframe>` artifacts?

## Implementation Map

### WorkspacePanel header (always visible)
```tsx
// In the artifact header row, after the filename and status
<div className="flex items-center gap-1 ml-2">
  <CopyButton text={activeArtifact.content} title="Copy markdown" label="MD" />
  <CaptureButton targetRef={contentRef} title="Copy as image" label="IMG" />
  <DownloadButton targetRef={contentRef} title="Download PNG" />
</div>
```

### PartnerMessage / dialogue bubble (hover reveal)
```tsx
// In each message bubble, positioned absolute top-right, opacity-0 group-hover:opacity-100
<button className="copy-action absolute top-1.5 right-1.5 p-1 rounded ..." title="Copy message">
  <Copy className="w-3 h-3" />
</button>
```

### New component: CaptureButton
```tsx
async function captureToClipboard(el: HTMLElement) {
  const { default: html2canvas } = await import('html2canvas')
  const canvas = await html2canvas(el, { useCORS: true, backgroundColor: null })
  canvas.toBlob(blob => {
    if (!blob) return
    navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      .catch(() => downloadCanvas(canvas)) // fallback
  })
}
```

## User Stories (final)

1. As a team member reviewing a spec, I want to copy the raw markdown with one click so I can paste it into Notion, email, or my editor.
2. As a product owner sharing progress in Slack, I want to copy the rendered artifact as an image so I can paste directly — no file, no cropping.
3. As someone giving feedback on a ponder conversation, I want to copy a single message bubble to quote it in a report or reply.

## MVP Scope

- WorkspacePanel: [MD] [IMG] [↓] button cluster in artifact header
- PartnerMessage: hover-reveal copy icon per bubble
- CaptureButton component: html2canvas + clipboard + download fallback
- CopyButton already exists — reuse with `label` prop added

## Not in scope (this iteration)
- PDF export
- Sharing to Slack/Discord directly
- HTML artifact screenshot (iframe limitation)
- Mobile long-press for dialogue copy (track as follow-up task)
