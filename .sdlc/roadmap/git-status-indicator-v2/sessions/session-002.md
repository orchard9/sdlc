---
session: 2
timestamp: 2026-03-07T23:55:00Z
orientation:
  current: "All three design artifacts complete — commit agent, hover tile, and directory grouping. Scope is tight and implementation-ready."
  next: "Run /sdlc-ponder-commit git-status-indicator-v2 to crystallize into milestone + features"
  commit: "Met — owner's brief is specific, all designs are concrete, code audit confirms feasibility"
---

**Xist - Owner**
group the file counts by top level directory. "6 .sdlc, 2 frontend, 1 crates" one per line, sort alphabetical.

---

**Facilitator**

Clear refinement to the hover tile design. Let me look at what we have to work with.

The porcelain v2 output already contains full file paths on every line:

```
1 .M N... 100644 ... .sdlc/changelog.yaml
1 .M N... 100644 ... crates/sdlc-server/src/lib.rs
? frontend/src/components/GitGreenQuote.tsx
```

The current parser (`git.rs:81-151`) walks every line to count dirty/staged/untracked/conflict — but throws away the file paths. We just need to also extract the first path component and aggregate.

### Backend change

Add `directory_counts: BTreeMap<String, u32>` to the `GitStatus` struct. In `parse_porcelain_v2`, for each file entry line:

- **Type 1 (ordinary):** path is the last space-separated field — `1 .M N... 100644 100644 100644 hash1 hash2 path/to/file`
- **Type 2 (rename):** path is after the tab — `2 R. N... 100644 100644 hash1 hash2 R100 old\tnew` (use the new path)
- **Type u (conflict):** similar to type 1
- **Type ? (untracked):** path is everything after `? `

Split on `/`, take first component. Accumulate in a `BTreeMap` (automatically sorted).

The response gains:
```json
{
  "directory_counts": {
    ".sdlc": 6,
    "crates": 2,
    "frontend": 1
  }
}
```

### Frontend change

The hover tile renders directory counts as a compact list:

```
.sdlc      6
crates     2
frontend   1
```

One line per directory, alphabetical (BTreeMap handles this server-side, `Object.entries().sort()` as a safety net client-side). Monospace-ish alignment with flexbox justify-between.

### Decisions

- **Decided:** Directory grouping counts ALL changed files per directory — dirty + staged + untracked + conflicted combined. Not separate buckets per status type. This matches the owner's ask ("file counts by top level directory") and keeps the tile scannable.
- **Decided:** Root-level files (no `/` in path) use the filename itself as the key. Rare in practice — most projects have no loose files at root.
- **Open:** Per-directory status breakdown (e.g., "3 .sdlc (2 modified, 1 untracked)") — defer to v3 if anyone asks.

### Session summary

This is a small, additive refinement. The hover tile design from session 1 showed flat counts; now it shows directory-grouped counts. No new API endpoints needed — just an extra field on the existing `/api/git/status` response. The scope remains tight: two required features (commit agent, hover tile with directory grouping) + one optional (file list API).

Commit signal remains met. Ready to crystallize.

**Next:** `/sdlc-ponder-commit git-status-indicator-v2`
