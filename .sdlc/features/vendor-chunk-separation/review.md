# Code Review: Vendor Chunk Separation

## Changes Reviewed

| File | Change |
|---|---|
| `frontend/vite.config.ts` | Added `build.rollupOptions.output.manualChunks` with three vendor groups |

## Findings

### 1. Ordering of regex checks — GOOD

The markdown group is checked first to prevent `react-markdown` from being captured by the react group. The react group uses path-delimited patterns (`/react/`, `/react-dom/`) for precision. This is correct and intentional.

### 2. Regex breadth for mermaid group — ACCEPTABLE

The mermaid regex matches `d3` broadly, which captures all `d3-*` sub-packages. This is correct since `d3-*` packages are transitive mermaid dependencies. If a non-mermaid feature later uses `d3` directly, the chunk assignment would still be correct (d3 would stay in the mermaid chunk).

### 3. Unmatched packages fall to default — GOOD

Packages not matched by any group (e.g., `lucide-react`, `tailwindcss`, `katex`) land in the default chunk. This is safe behavior — no risk of missing dependencies.

### 4. Build output verified

- `vendor-react`: 229 KB (gzip 73 KB)
- `vendor-markdown`: 157 KB (gzip 47 KB)
- `vendor-mermaid`: 2,449 KB (gzip 689 KB)
- Main `index`: 159 KB (gzip 44 KB)
- No circular chunk warnings
- Clean build with exit code 0

### 5. All existing tests pass

22 tests across 3 test files pass without modification.

## Verdict

Clean, minimal change. Single file modified. No regressions. Approved.
