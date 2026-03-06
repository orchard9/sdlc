# Spec: Fix Scrollbar Styling (Hidden/Clean/Modern)

## Problem

`frontend/src/index.css` has zero scrollbar CSS rules — no `::-webkit-scrollbar` pseudo-elements and no `scrollbar-width`/`scrollbar-color` properties. The browser renders native OS scrollbars on every scrollable surface throughout the app. These clash with the dark minimal design aesthetic.

Affected surfaces: sidebars, panels, modals, log views, markdown content areas, and any element with `overflow: auto` or `overflow: scroll`.

## Goal

Apply global scrollbar styling in `index.css` so that:
1. On Webkit/Blink (Chrome, Safari, Edge) scrollbars are visually minimal — thin, dark-colored, matching the design palette.
2. On Firefox scrollbars are thin with matching colors via `scrollbar-width: thin` and `scrollbar-color`.
3. No native OS chrome remains visible anywhere in the app.

## Solution

Add a scrollbar styling block to `frontend/src/index.css` immediately after the `body` rule:

```css
/* ── Scrollbars ────────────────────────────────────────────────── */

/* Firefox */
* {
  scrollbar-width: thin;
  scrollbar-color: oklch(0.35 0 0) transparent;
}

/* Webkit / Blink */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: oklch(0.35 0 0);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: oklch(0.45 0 0);
}
::-webkit-scrollbar-corner {
  background: transparent;
}
```

Colors are sourced from the existing design palette (between `--color-border: oklch(0.3 0 0)` and `--color-secondary: oklch(0.25 0 0)`), keeping thumb visible but subtle.

## Acceptance Criteria

- `index.css` contains both `scrollbar-width`/`scrollbar-color` (Firefox) and `::-webkit-scrollbar` rules (Webkit).
- No additional component files need modification — global rules cover all scrollable surfaces.
- Scrollbar thumb color blends with the dark background and is clearly distinct from the track.
- Scrollbar width is 6 px or less.

## Out of Scope

- Per-component scrollbar overrides.
- Hiding scrollbars entirely (must remain accessible/discoverable on scroll).
