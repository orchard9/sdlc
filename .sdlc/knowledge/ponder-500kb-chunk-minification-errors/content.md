---
session: 1
timestamp: 2026-03-08T00:10:00Z
orientation:
  current: "Root-caused the 2.3MB index chunk — zero code splitting, three heavy libraries inlined"
  next: "Implement manualChunks + lazy routes in vite.config.ts and App.tsx"
  commit: "Solution is clear and mechanical — ready to commit now"
---

**Xist · Owner**
500kb-chunk-minification-errors

when running just install, this error occurs:
(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.

---

## Investigation

Ran `npx vite build` and captured the full output. Key findings:

### The monster chunk

`index-DHFdrhfg.js` is **2,373 kB** (690 kB gzipped). This is the single entry chunk containing *everything* — all 25 pages, all components, all libraries.

### Why it's so big — two root causes

**1. Zero route-level code splitting.** All 25 pages are statically imported in `App.tsx` (lines 6–30):
```tsx
import { Dashboard } from '@/pages/Dashboard'
import { FeatureDetail } from '@/pages/FeatureDetail'
// ... 23 more static imports
```
Every page and its transitive dependencies land in the index chunk.

**2. Heavy libraries with no manual chunking.** Three libraries dominate the non-mermaid weight:
- **react-syntax-highlighter** — ships all 180+ language grammars by default. Used only in `MarkdownContent.tsx`. Contributes hundreds of kB.
- **react-markdown + remark-gfm** — used in 5 components but pulled into the main chunk
- **mermaid** — Vite auto-splits its diagram sub-modules (good), but the core still lands in index

Mermaid's auto-split diagrams are already reasonable (treemap 452kB, cytoscape 441kB, katex 261kB as separate chunks) — Vite handles those. The problem is everything *else* piled into index.

### What's NOT the problem

- CSS is fine: `index.css` at 100 kB
- Mermaid diagram sub-chunks are already split (Vite async imports handle this)
- The build itself succeeds — this is a warning, not an error

## Analysis — Three fixes, priority ordered

### Fix 1: Lazy routes (biggest impact, simplest change)

Convert all static page imports to `React.lazy()`:

```tsx
const Dashboard = lazy(() => import('@/pages/Dashboard'))
const FeatureDetail = lazy(() => import('@/pages/FeatureDetail'))
// etc.
```

Wrap routes in `<Suspense>`. Each page becomes its own chunk. The index chunk drops from 2.3MB to roughly the size of shared components + React + router.

**Impact estimate:** Splits ~25 page-level chunks. Index chunk drops to ~300-500kB depending on shared deps.

### Fix 2: manualChunks for vendor libraries

Add to `vite.config.ts`:

```ts
build: {
  rollupOptions: {
    output: {
      manualChunks: {
        'vendor-react': ['react', 'react-dom', 'react-router-dom'],
        'vendor-markdown': ['react-markdown', 'remark-gfm', 'react-syntax-highlighter'],
        'vendor-mermaid': ['mermaid'],
      }
    }
  }
}
```

This prevents vendor churn — when app code changes, vendor chunks stay cached. Also keeps the heavy markdown/syntax-highlighter bundle (~400kB) separate and only loaded by pages that render markdown.

**Impact estimate:** Stable vendor caching, ~3 predictable vendor chunks instead of one mega-bundle.

### Fix 3: Lighter syntax highlighting import

`react-syntax-highlighter` has a `/dist/esm/light` build that only includes languages you register. Currently importing the full build:

```tsx
import SyntaxHighlighter from 'react-syntax-highlighter'
```

Switch to:
```tsx
import { Light as SyntaxHighlighter } from 'react-syntax-highlighter'
import js from 'react-syntax-highlighter/dist/esm/languages/hljs/javascript'
import rust from 'react-syntax-highlighter/dist/esm/languages/hljs/rust'
// register only needed languages
SyntaxHighlighter.registerLanguage('javascript', js)
SyntaxHighlighter.registerLanguage('rust', rust)
```

**Impact estimate:** Drops syntax-highlighter from ~300kB to ~30kB.

## Decision

? Open: Do we do all three, or just the first two?

Fix 3 (light syntax highlighter) has a UX trade-off — we'd need to enumerate which languages to support. Given that this is a dev tool rendering arbitrary markdown from agents, we probably want broad language support. Could register the top 10-15 languages and still save 200kB+.

**Decided:** Fixes 1 and 2 are pure wins with no trade-offs. Fix 3 is optional.

? Open: Should we suppress the warning with `chunkSizeWarningLimit`?

**Decided:** No. The warning is useful — it caught a real problem. After fixes 1+2, if individual vendor chunks still exceed 500kB (mermaid will), that's fine and expected for a chunked vendor lib. We can raise the limit to 600kB at that point if the warning is noisy, but not to hide the problem.

## Commit signal

This is a mechanical refactoring task — the solution is well-understood, the implementation is straightforward, and there are no architectural trade-offs to debate. Ready to commit as a milestone with a single feature.
