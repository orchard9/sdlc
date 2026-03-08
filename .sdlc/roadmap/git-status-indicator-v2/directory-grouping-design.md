# Directory-Grouped File Counts

## Requirement

The hover tile should show file counts grouped by top-level directory, one per line, sorted alphabetically:

```
3 .sdlc
1 crates
1 frontend
```

Not flat counts like "5 modified". The grouping tells you **where** the changes are, not just how many.

## Backend: extend `GitStatus` response

The porcelain v2 output already contains full file paths:
```
1 .M N... 100644 ... .sdlc/changelog.yaml
1 .M N... 100644 ... crates/sdlc-server/src/lib.rs
? frontend/src/components/GitGreenQuote.tsx
```

Add a new field to the API response:

```json
{
  "directory_counts": {
    ".sdlc": 6,
    "crates": 2,
    "frontend": 1
  }
}
```

Implementation in `parse_porcelain_v2`:
- For each `1 `, `2 `, `u `, and `? ` line, extract the file path (last field for type 1/u, tab-separated for type 2 renames, after `? ` for untracked)
- Split on `/` and take the first component as the directory key
- Files at root (no `/`) use the filename itself as the key
- Aggregate into a `BTreeMap<String, u32>` (BTreeMap for alphabetical ordering)

## Frontend: render in hover tile

```tsx
{Object.entries(status.directory_counts)
  .sort(([a], [b]) => a.localeCompare(b))
  .map(([dir, count]) => (
    <div key={dir} className="flex justify-between text-xs">
      <span className="text-muted-foreground">{dir}</span>
      <span>{count}</span>
    </div>
  ))}
```

## Decision

- Decided: Directory grouping counts ALL changed files (dirty + staged + untracked + conflicted) per directory. Not separate buckets per status.
- Open: Should the hover tile also show per-directory breakdown by status (e.g., "3 .sdlc (2 modified, 1 untracked)")? Start simple — total count only.