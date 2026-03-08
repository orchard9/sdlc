# Tasks: Vendor Chunk Separation

## T1: Add manualChunks to vite.config.ts

Add `build.rollupOptions.output.manualChunks` function with three vendor groups: `vendor-react`, `vendor-markdown`, `vendor-mermaid`.

## T2: Verify build output

Run `npm run build` and verify that separate chunk files are produced for each vendor group. Confirm no build warnings or errors.

## T3: Verify application loads correctly

Confirm the built application loads and renders without errors — all pages accessible, no missing module errors in console.
