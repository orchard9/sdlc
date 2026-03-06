# Specification: Artifact and Dialogue Copy & Screenshot

## Problem

The sdlc UI renders artifacts and ponder dialogues beautifully but provides no path to extract that content for sharing. Users resort to OS-level screenshot keybindings (memorized by heavy users) and manual text selection. This friction compounds in exactly the moments that matter: reviewing a spec, sharing a ponder session, providing feedback on a design.

## Scope

Add one-click copy and image capture to the two primary content surfaces:

1. **WorkspacePanel** — the artifact viewer used in ponder, feature detail, and investigation pages
2. **PartnerMessage / SessionBlock** — individual dialogue message bubbles in ponder sessions

## User Stories

1. As a team member reviewing a spec, I want to copy the raw markdown with one click so I can paste it into Notion, email, or my editor with formatting intact.
2. As a product owner sharing progress in Slack, I want to copy the rendered artifact as an image so I can paste directly — no file, no attachment, no cropping.
3. As someone giving feedback on a ponder session, I want to copy a single message bubble so I can quote it in a report, reply, or feedback thread.

## Surfaces

### WorkspacePanel — artifact header buttons (always visible)

When an artifact is active and open, show a button cluster in the artifact header row:

- **[MD]** — copy raw markdown text to clipboard. Uses existing `CopyButton` with a `label="MD"` prop. 2-second green "Copied" flash on success.
- **[IMG]** — copy rendered artifact as PNG image to clipboard. Uses new `CaptureButton`. Brief "Capturing…" state, then 2-second green "Copied" flash. Falls back to download if clipboard write fails.
- **[↓]** — download rendered artifact as PNG file. Icon-only, secondary affordance.

**HTML artifact exception:** When the active artifact is `.html` or `.htm`, the artifact panel renders it in an `<iframe srcDoc>`. `html2canvas` cannot capture iframes. For HTML artifacts, show [MD] only — [IMG] and [↓] are hidden.

### PartnerMessage / SessionBlock — dialogue bubbles (hover reveal)

Each message bubble in a ponder dialogue session gets a copy affordance:

- Wrap the bubble in a `group` container
- Add an absolutely-positioned copy icon button in the top-right corner
- Desktop: `opacity-0 group-hover:opacity-100 transition-opacity` (hover reveal)
- Touch/mobile: always visible (hover-reveal is not available on touch)

Clicking copies the message text content to clipboard. Uses the same 2-second check flash.

## Technical Decisions

| Decision | Choice |
|---|---|
| Copy markdown | `navigator.clipboard.writeText(artifact.content)` — existing `CopyButton` primitive |
| Copy as image | `html2canvas` (dynamic import) → `ClipboardItem({ 'image/png': blob })` |
| Download PNG | Same html2canvas capture → `canvas.toDataURL()` → `<a>` click |
| Bundle impact | Dynamic import — html2canvas not in initial bundle |
| Clipboard fail | Auto-fallback to download — never silently fail |
| HTML iframes | Hide image buttons — not a graceful degradation, just absent |
| Mobile touch | Always-visible copy icon via `window.matchMedia('(pointer: coarse)')` |

## New Component: CaptureButton

`frontend/src/components/shared/CaptureButton.tsx`

Props:
- `targetRef: RefObject<HTMLElement>` — element to capture
- `mode: 'copy' | 'download'` — what to do with the canvas
- `label?: string` — optional text label (e.g. "IMG")
- `filename?: string` — for download mode, default `'artifact.png'`
- `title?: string` — tooltip

States: `idle` → `capturing` → `done` (2s) → `idle`

Error handling: if `mode='copy'` and `clipboard.write` throws, fall back to download.

## Out of Scope

- PDF export
- Direct share to Slack/Discord
- Long-press context menu for mobile dialogue copy (tracked as a follow-up task)
- Screenshot of HTML artifact preview (iframe limitation, documented in code)
