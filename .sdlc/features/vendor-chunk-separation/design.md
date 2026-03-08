# Design: Vendor Chunk Separation

## Approach

Add a `manualChunks` function to `frontend/vite.config.ts` under `build.rollupOptions.output`. The function inspects each module's `id` (its resolved file path) and routes `node_modules` imports into named vendor chunks.

## Chunk Groups

```
manualChunks(id) {
  if (id.includes('node_modules')) {
    // React core ecosystem
    if (id.match(/react|react-dom|react-router|scheduler/))
      return 'vendor-react'

    // Markdown processing pipeline
    if (id.match(/react-markdown|remark|rehype|unified|mdast|micromark|hast|unist|vfile|bail|trough|property-information|comma-separated|space-separated|decode-named|character-entities|ccount|escape-string|devlop|stringify-entities/))
      return 'vendor-markdown'

    // Mermaid diagramming
    if (id.match(/mermaid|dagre|d3|khroma|cytoscape|elkjs|dompurify|lodash|stylis/))
      return 'vendor-mermaid'
  }
}
```

## File Changes

| File | Change |
|---|---|
| `frontend/vite.config.ts` | Add `build.rollupOptions.output.manualChunks` |

## Rationale

- **Three groups** align with the three heaviest dependency clusters in the bundle
- Matching on path substrings within `node_modules` is the standard Vite/Rollup pattern
- Unmatched `node_modules` packages remain in the default chunk — no risk of missing dependencies
- The regex patterns are intentionally broad to capture transitive dependencies (e.g., `d3-*` for mermaid)

## Risks

- A future dependency upgrade could introduce a new transitive dependency not matched by any group — it would simply land in the default chunk, which is acceptable
- Over-broad regex could pull unrelated packages — mitigated by testing the build output
