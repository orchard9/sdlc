# Tasks: Fix Scrollbar Styling

## T1 — Add global scrollbar CSS rules to index.css

Add Firefox (`scrollbar-width`/`scrollbar-color`) and Webkit (`::-webkit-scrollbar` pseudo-elements) scrollbar styling to `frontend/src/index.css` immediately after the `body` rule block.

**Done when:** `index.css` contains both rule sets and a browser renders minimal styled scrollbars on all scrollable surfaces.
