# Spec: sdlc ponder merge — CLI command and core data model

## Overview

Add a `sdlc ponder merge <source> --into <target>` command that consolidates two ponder entries into one. Sessions are copied (renumbered), artifacts are copied (prefixed if colliding), team members are merged, and the source entry is parked with a forwarding pointer. This keeps the roadmap clean as the idea space grows.

## Motivation

As projects evolve, related or duplicate ponder entries accumulate. Agents and humans need a single command to consolidate them without losing any session history or scrapbook artifacts.

## Scope

### In scope
- Core data model: `merged_into` field on PonderEntry, `merged_from` Vec on PonderEntry
- `sdlc ponder merge <source> --into <target>` CLI command
- Session copying with renumbering and merge-origin header comment
- Artifact copying (skip manifest.yaml, team.yaml; prefix on collision)
- Team member dedup merge
- Source entry parked with `merged_into` pointer
- Target entry updated with `merged_from` list, aggregated tags, bumped session count
- `sdlc ponder show` displays redirect banner for merged entries
- `sdlc ponder list` hides merged entries by default; `--all` flag shows them with indicator
- Pre-condition validation: reject merge if source or target is Committed

### Out of scope
- REST API endpoints (separate feature if needed)
- Frontend UI for merge
- Multi-source merge in a single invocation (user can run merge repeatedly)
- Undo/unmerge

## Data Model Changes

### PonderEntry (ponder.rs)

Add two optional fields:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub merged_into: Option<String>,       // slug of target entry (set on source after merge)

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub merged_from: Vec<String>,          // slugs of source entries (accumulated on target)
```

Both fields are backward-compatible via `serde(default)`.

## CLI Interface

```
sdlc ponder merge <source-slug> --into <target-slug> [--json]
```

### Behavior

1. **Validate pre-conditions**
   - Both entries must exist
   - Source must not be Committed
   - Target must not be Committed
   - Source must not already have `merged_into` set (no double-merge)

2. **Copy sessions** from source to target
   - Read all source session files from `.sdlc/roadmap/<source>/sessions/`
   - For each, prepend a header comment: `<!-- merged from: <source-slug>, original session: N -->`
   - Write to target sessions dir with next available session numbers
   - Update target's session counter

3. **Copy artifacts** from source to target
   - List source artifacts (skip manifest.yaml, team.yaml, sessions/ dir)
   - For each file, copy to target dir
   - If filename already exists in target, prefix with `<source-slug>--` to avoid collision

4. **Merge team members**
   - Load both teams
   - Add source team members to target team (skip duplicates by name)

5. **Update target manifest**
   - Add source slug to `merged_from`
   - Merge tags (union, deduplicated)
   - Bump session count by number of copied sessions
   - Set `updated_at`

6. **Park source**
   - Set source `status: parked`
   - Set source `merged_into: <target-slug>`
   - Remove source from `active_ponders` in state.yaml

7. **Output**
   - Human: `Merged '<source>' into '<target>': N sessions, M artifacts, K team members copied`
   - JSON: `{ "source": "...", "target": "...", "sessions_copied": N, "artifacts_copied": M, "team_members_copied": K }`

## Display Changes

### `sdlc ponder show <slug>`
When the entry has `merged_into` set, display a banner:
```
⚠ This entry was merged into '<target>'. Use `sdlc ponder show <target>` instead.
```
Then show the entry details as normal.

### `sdlc ponder list`
- Default: hide entries where `merged_into` is Some (they are parked redirects)
- `--all` flag: show all entries; merged entries display status as `parked -> <target>`

## Error Cases

| Condition | Error message |
|---|---|
| Source not found | `ponder entry '<slug>' not found` |
| Target not found | `ponder entry '<slug>' not found` |
| Source is Committed | `cannot merge committed entry '<slug>': uncommit first` |
| Target is Committed | `cannot merge into committed entry '<slug>'` |
| Source already merged | `'<slug>' was already merged into '<target>'` |
| Source == Target | `cannot merge an entry into itself` |

## Testing

- Unit tests in `ponder.rs` for the merge data model (merged_into/merged_from serde roundtrip)
- Unit test for merge_entries core function: sessions copied, artifacts copied, team merged, source parked
- Unit test for pre-condition validation (committed entries rejected, self-merge rejected)
- Integration test via CLI: full merge command with verification of file system state
