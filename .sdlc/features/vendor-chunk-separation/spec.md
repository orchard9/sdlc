# Spec: Vendor Chunk Separation

## Problem

The frontend currently bundles all vendor dependencies into a single chunk alongside application code. This means any application code change invalidates the entire bundle cache, forcing users to re-download large vendor libraries (React, react-markdown, mermaid) that haven't changed.

## Solution

Add Vite `build.rollupOptions.output.manualChunks` configuration to split vendor dependencies into separate, stable chunks:

1. **react-vendor** — `react`, `react-dom`, `react-router-dom` and related React ecosystem packages
2. **markdown-vendor** — `react-markdown`, `remark-*`, `rehype-*`, `unified`, `mdast-*`, `micromark-*` and related markdown processing packages
3. **mermaid-vendor** — `mermaid` and its transitive dependencies

## Acceptance Criteria

- `npm run build` produces separate chunk files for each vendor group
- Each vendor chunk contains only the packages assigned to its group
- Application code changes do not alter vendor chunk hashes
- The application loads and renders correctly with the split chunks
- No increase in total number of HTTP requests beyond the three new vendor chunks
- Build completes without warnings or errors

## Out of Scope

- Lazy route splitting (handled by `lazy-route-splitting` feature)
- Dynamic import() of application pages
- CDN or external module federation
- SSR or server-side chunk optimization
