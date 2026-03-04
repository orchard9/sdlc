# Spec: Scrapbook renders *-mockup.html artifacts in an inline iframe

## Problem

When an agent captures a `*-mockup.html` artifact into a ponder scrapbook, the
WorkspacePanel lists it as a file with a file icon — identical to Markdown files.
Users must click to expand it, and the expanded panel uses `max-h-64` (256 px),
which crops most mockups. There is no visual cue that the artifact is a rendered
HTML preview.

This creates friction in the design review flow: the whole point of HTML mockups
is that reviewers can *see* the design immediately, but the current UI hides them
behind a tiny constrained panel.

## What We're Building

Improve the `WorkspacePanel` and `ArtifactContent` components so that
`*-mockup.html` artifacts (and any `.html`/`.htm` file) are treated as
**visual preview artifacts** with distinct UX:

1. **Artifact list row** — `.html` files show a monitor/globe icon instead of the
   generic `FileText` icon, and display a `Preview` badge so users know they are
   interactive previews.

2. **Expanded content panel** — When a `.html` artifact is active, the inline
   expansion panel uses a taller constrained height (`max-h-96`, 384 px) instead
   of `max-h-64` (256 px), giving mockups more breathing room without dominating
   the panel.

3. **Fullscreen experience** — Already works well (`min-h-[60vh]`). No change
   needed; the Maximize button remains the canonical path for full review.

4. **`ArtifactContent` iframe sizing** — The `fullscreen=false` iframe uses
   `min-h-[300px] max-h-96` (was `min-h-64 max-h-80`) to match the taller
   expansion panel and avoid double-scrollbar confusion inside the panel.

No backend changes are required. The API already returns `content` for all
artifacts. The changes are entirely in the frontend components.

## Scope

### `frontend/src/components/shared/ArtifactContent.tsx`

- Change the non-fullscreen iframe class from `'min-h-64 max-h-80'` to
  `'min-h-[300px] max-h-96'`.

### `frontend/src/components/ponder/WorkspacePanel.tsx`

- Import `Monitor` (or `Globe`) from `lucide-react` alongside the existing icons.
- In the artifact list row, render:
  - `Monitor` icon for `.html`/`.htm` files (instead of `FileText`).
  - A small `Preview` text badge (e.g., `bg-primary/10 text-primary text-[10px]
    px-1 rounded`) next to the filename for `.html`/`.htm` files.
- Change the expanded content panel's height constraint from `max-h-64` to
  `max-h-96` so the iframe has room to render.

## Acceptance Criteria

1. In `ArtifactContent`, the non-fullscreen HTML iframe uses `min-h-[300px]
   max-h-96`.

2. In `WorkspacePanel`, `.html`/`.htm` artifact rows display a `Monitor` icon
   (not `FileText`) and a small `Preview` badge inline with the filename.

3. The expanded content panel for an active `.html`/`.htm` artifact uses
   `max-h-96` (not `max-h-64`).

4. Markdown and other artifact types are visually unchanged.

5. `SDLC_NO_NPM=1 cargo test --all` passes (no Rust changes involved).

6. `npm run build` in `frontend/` succeeds with no TypeScript errors.

## Non-Goals

- Auto-expanding HTML artifacts without a click (would break the list layout
  for entries with many artifacts).
- Rendering mockups in the artifact list thumbnail — this is out of scope and
  would require a different architecture.
- Any backend / Rust changes — this is a pure frontend change.
- Resizable iframe — the fullscreen button is the correct path for detailed review.
