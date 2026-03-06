# Plan: Artifact Share & Capture

## Milestone: v41-artifact-share-capture

**Vision:** Users can copy and capture any artifact or dialogue message in one click — no OS-level screenshot keybindings, no manual text selection, no manual cropping. Share specs to Slack, quote ponder messages in feedback, extract rendered views as images.

**User-observable goal:** Open any artifact. Click [MD] to copy its markdown. Click [IMG] to copy it as an image ready to paste into Slack or email. In any ponder session, hover a message and click the copy icon. Done.

## Acceptance criteria

- `WorkspacePanel` shows [MD] [IMG] [↓] buttons in the artifact header when an artifact is active
- Clicking [MD] copies the raw markdown text to clipboard (2s green flash)
- Clicking [IMG] copies the rendered artifact as a PNG image to clipboard (2s green flash)
- Clicking [↓] downloads the artifact as a PNG file
- Hovering a `PartnerMessage` dialogue bubble reveals a copy icon in the top-right corner
- Clicking the copy icon copies the message text to clipboard
- On mobile/touch, the copy icon is always visible on dialogue bubbles (hover-reveal not available)
- HTML artifacts (iframe-rendered) show [MD] only — [IMG] and [↓] are hidden/disabled with a tooltip explaining the limitation
- If clipboard image write fails (permissions), [IMG] falls back to download automatically
- html2canvas is dynamically imported — no impact on initial bundle size

## Feature

### artifact-copy-capture — Artifact and Dialogue Copy & Screenshot

**Why:** Jordan and team members share artifact content constantly — specs to reviewers, ponder sessions to stakeholders, design mockups to Discord. Currently this requires OS screenshot keybindings (memorized by heavy users) or manual text selection. A one-click path directly reduces friction and increases feedback quality from all users, not just power users.

**Tasks:**

1. **Add label prop to CopyButton**
   Extend `CopyButton.tsx` to accept an optional `label?: string` prop. When provided, render it as a text label next to the icon (e.g. "MD"). When absent, icon-only (backward compatible with all existing usages in `CommandBlock`).

2. **Create CaptureButton component**
   New `frontend/src/components/shared/CaptureButton.tsx`. Wraps html2canvas with dynamic import. Two modes: `mode="copy"` (ClipboardItem) and `mode="download"` (anchor click). Copy mode falls back to download if clipboard write is denied. States: idle → capturing → copied/failed → idle. Uses same visual language as CopyButton (check icon on success).

3. **Add copy/screenshot buttons to WorkspacePanel artifact header**
   In `WorkspacePanel.tsx`, when `activeArtifact` is non-null, add a ref to the content div and render the button cluster in the artifact header row: `<CopyButton label="MD" text={activeArtifact.content} />` + `<CaptureButton mode="copy" label="IMG" targetRef={contentRef} />` + `<CaptureButton mode="download" targetRef={contentRef} />`. For HTML artifacts (`.html`/`.htm` extension), render [MD] only — skip [IMG] and [↓] with a `title="Screenshot not available for HTML previews"` placeholder.

4. **Add hover-reveal copy button to PartnerMessage**
   In `PartnerMessage.tsx` (and equivalent plain message rendering in `SessionBlock`), wrap the message bubble in a `group` div. Add an absolutely-positioned `<CopyButton>` icon in the top-right corner with `className="opacity-0 group-hover:opacity-100 transition-opacity"`. CopyButton receives the message text content.

5. **Mobile fallback for dialogue copy**
   Add a `useTouchDevice` hook or media query check. On touch-capable devices, render the copy icon with `opacity-100` (always visible) instead of hover-reveal. Small icon, doesn't interrupt reading flow.

6. **Graceful HTML artifact handling**
   When the active artifact is `.html`/`.htm`, the image capture buttons are hidden (not disabled with grey state — just absent). Add a tooltip on the [IMG] slot explaining why. Document this as a known limitation in a code comment.
