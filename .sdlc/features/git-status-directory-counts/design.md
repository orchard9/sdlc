# Design: Per-directory file counts in git status API

## Approach

This is a backend-only change to `crates/sdlc-server/src/routes/git.rs`. No frontend or UI changes are in scope.

## Data Model

### New struct

```rust
#[derive(Debug, Serialize, Clone)]
pub struct DirectoryCount {
    pub directory: String,
    pub count: u32,
}
```

### Extended GitStatus

Add one field to the existing `GitStatus` struct:

```rust
pub directory_counts: Vec<DirectoryCount>,
```

## Parsing Changes

The `parse_porcelain_v2` function already iterates every line of `git status --porcelain=v2 --branch` output. The change adds file-path extraction during the same loop:

```
Line type     | Path extraction
------------- | -----------------------------------------------
"1 XY ..."    | Last space-separated token (field 9+)
"2 XY ..."    | Destination path after tab separator
"? path"      | Everything after "? "
"u XY ..."    | Last space-separated token (field 11+)
```

A `HashMap<String, u32>` accumulates counts keyed by the immediate parent directory of each path. The parent is derived by finding the last `/` in the path string:
- Path `"a/b/c.rs"` -> directory `"a/b"`
- Path `"file.rs"` -> directory `"."`

After the parsing loop, the HashMap is converted to a `Vec<DirectoryCount>`, sorted descending by `count`.

## Function Signatures

No new public functions. The change is internal to `parse_porcelain_v2` (which gains a `HashMap` accumulator) and the `GitStatus` struct (which gains the `directory_counts` field).

One small private helper:

```rust
fn parent_directory(path: &str) -> &str
```

Returns everything before the last `/`, or `"."` if no slash is present.

## Backward Compatibility

The `directory_counts` field is purely additive. All existing fields and their computation are unchanged. Existing frontend consumers (`useGitStatus`, `GitStatusChip`) ignore unknown fields and are unaffected.

## Test Plan

Unit tests added to the existing `mod tests` in `git.rs`:

1. **Clean repo** — `directory_counts` is empty.
2. **Single directory** — one dirty file produces one entry.
3. **Multiple directories** — files across three directories produce three entries sorted by count.
4. **Root-level files** — files without a `/` in the path grouped under `"."`.
5. **Renamed files** — destination path used for grouping.
6. **Mixed types** — dirty + staged + untracked in the same directory combine into one count.
