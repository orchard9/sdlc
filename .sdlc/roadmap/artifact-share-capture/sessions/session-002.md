---
session: 2
timestamp: 2026-03-04T00:00:00Z
orientation:
  current: "Problem shaped — clear user stories, implementation map, component design decided. Mockup produced."
  next: "Commit to a milestone — this is a small, self-contained feature ready for implementation"
  commit: "All decisions made, implementation map clear, mockup in scrapbook — ready"
---

## Session 2: Deep Exploration

### Codebase audit

Read `CopyButton.tsx`, `CommandBlock.tsx`, `WorkspacePanel.tsx`, `SessionBlock.tsx`, `ArtifactContent.tsx`, `UnifiedDialoguePanel.tsx`, `PartnerMessage.tsx`.

Key finding: `CopyButton` already exists and is solid. The copy primitive is done. What's missing is its presence on the two major content surfaces: `WorkspacePanel` (artifact viewer) and `SessionBlock`/`PartnerMessage` (dialogue messages).

### Recruited thought partners

Recruited: Maya Chen · UX Engineer (Linear, Notion background)
Recruited: Tavish Reid · Platform Engineer (browser APIs, html2canvas)
Recruited: Priya Nair · Product Skeptic

### Partner voices

**Maya Chen · UX Engineer**
Placement matters. Hover-reveal on dialogue messages keeps visual noise low — good. Artifact header buttons should be always-visible — users are in intentional "view this artifact" mode, no clutter argument. Make the screenshot button scannable, not hidden behind a tooltip.

**Tavish Reid · Platform Eng**
html2canvas is the right call. Handles CSS custom properties better than dom-to-image. Dynamic import to avoid bundle bloat. iframe limitation is real — `ArtifactContent` with HTML files renders in `<iframe srcDoc>`, html2canvas won't capture it. Acceptable tradeoff: gracefully disable or hide image buttons for HTML artifacts.

ClipboardItem path: `navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])`. Falls back to download if clipboard-write permission denied.

**Priya Nair · Product Skeptic**
Pushed on: "is screenshot solving a real problem?" Answer: yes — existing notification channels share state, not visual context. A screenshot preserves the conversation tone, the rendering, the moment. Agreed to build. But: make "copy as image" primary over "download" — people want to paste directly into Slack, not attach a file.

⚑  Decided: primary action is copy-to-clipboard-as-image, download is secondary fallback.

### Decisions made

⚑  Copy markdown → existing `CopyButton` with `artifact.content`. Zero new deps.
⚑  Copy as image → html2canvas (dynamic import) → `ClipboardItem({ 'image/png': blob })`
⚑  Download PNG → same capture, different output. Secondary/fallback.
⚑  Artifact buttons: always visible in artifact header.
⚑  Dialogue copy: hover-reveal per message bubble.
⚑  Bundle impact: dynamic import so html2canvas doesn't inflate initial load.
⚑  Error fallback: clipboard fail → download. Never silently fail.
⚑  No PDF, no direct Slack share — out of scope for MVP.

?  Open: Mobile UX for dialogue hover — touch doesn't have hover. Either always-visible or long-press.
?  Open: HTML artifact button state — disable image buttons for iframe artifacts, or hide them?

### Mockup produced

`artifact-share-capture-mockup.html` — 3-state prototype:
- State A: Artifact viewer with [MD] [IMG] [↓] button cluster in header
- State B: Dialogue session with hover-reveal copy icons per message
- State C: All feedback states (copied / capturing / failed / download fallback)

### Implementation map

Captured in `exploration.md`. Summary:
- `WorkspacePanel` header: add `CopyButton` + new `CaptureButton` + `DownloadButton`
- `PartnerMessage` bubbles: absolute-positioned copy icon, `opacity-0 group-hover:opacity-100`
- New `CaptureButton` component: wraps html2canvas dynamic import + clipboard + download fallback
- `CopyButton` may need `label` prop added (currently just icon) for "MD" / "IMG" text label variant

## Product Summary

### What we explored
How to add one-click copy and screenshot buttons to the sdlc UI, so users can share artifact content and dialogue messages without OS-level screenshot tools.

### Key shifts
Before this session, the feature was just a request. After: the implementation is fully mapped. We found that the copy primitive already exists (`CopyButton`), the only new work is placement + a new `CaptureButton` component for html2canvas-based image capture. The scope is smaller than expected.

### Implications
This is a small, focused feature — a day or two of implementation. No architectural changes. Delivers high user value (Jordan's explicitly frequent pain point). Also lowers the feedback barrier for less technical users, which improves product signal quality over time.

### Still open
- Mobile UX for dialogue hover: should copy icon always show on mobile, or require long-press?
- HTML artifact buttons: disable or hide image capture for iframe-rendered HTML files?
