## `sdlc ponder merge` — Design Sketch

### CLI
```bash
sdlc ponder merge <source> --into <target>
```

### Data model additions to `PonderEntry`
```yaml
merged_into: <target-slug>      # on source (parked)
merged_from:                    # on target (list — can absorb multiple)
  - <source-slug>
```

### Behavior

**Pre-conditions:**
- Source and target exist
- Source is not already `parked` with a `merged_into`
- Source ≠ target
- Target is not parked

**File operations (non-destructive — copy, not move):**

1. **Sessions** — copy source session files to target, renumbered from `(target.sessions + 1)`; each gets a merge header comment
2. **Artifacts** — copy scrapbook files; if filename collides, use `from-<source>-<filename>` prefix
3. **Team** — union-merge `team.yaml` by `agent` field

**Manifest updates:**
- Source: `status: parked`, `merged_into: <target>`
- Target: append `merged_from: [<source>]`

**CLI output:**
```
Merged 'source-slug' → 'target-slug'
Sessions copied:   N
Artifacts copied:  N (filenames)
Team merged:       N members
Source is now:     parked (merged_into: target-slug)
```

### Files to touch (v1)
| File | Change |
|---|---|
| `sdlc-core/src/ponder.rs` | Add `merged_into`, `merged_from` fields; add `merge()` fn |
| `sdlc-cli/src/cmd/ponder.rs` | Add `Merge` subcommand + handler |
| `sdlc ponder show` | Display `merged_into` / `merged_from` when set |

### Out of scope for v1
- Server route (`/api/ponder/merge`)
- UI banner
- `sdlc ponder unmerge` (git is the undo button)
- `--link-only` flag
