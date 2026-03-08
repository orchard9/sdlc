# Solution Plan

## Fix 1: Lazy Routes (primary fix)
Convert all 25 static page imports in `App.tsx` to `React.lazy()` + `<Suspense>`.
Expected impact: index chunk drops from 2,373kB to ~300-500kB.

## Fix 2: Manual Vendor Chunks
Add `build.rollupOptions.output.manualChunks` to `vite.config.ts`:
- `vendor-react`: react, react-dom, react-router-dom
- `vendor-markdown`: react-markdown, remark-gfm, react-syntax-highlighter
- `vendor-mermaid`: mermaid

## Fix 3 (optional): Light Syntax Highlighter
Switch to `react-syntax-highlighter/dist/esm/light` build with registered languages.
Trade-off: must enumerate supported languages.

## Files to Change
- `frontend/src/App.tsx` — lazy imports + Suspense wrapper
- `frontend/vite.config.ts` — manualChunks config
- (optional) `frontend/src/components/shared/MarkdownContent.tsx` — light SyntaxHighlighter