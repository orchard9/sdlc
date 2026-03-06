# Spec: ponder list/show UX for merged entries -- filter and redirect banner

## Overview

When a ponder entry has been merged into another (via `sdlc ponder merge`), the list and show views across CLI, REST API, and frontend should communicate this clearly. Merged entries are hidden by default from lists and display a redirect banner when shown individually.

## Motivation

After `ponder-merge-cli` adds `merged_into` / `merged_from` fields to `PonderEntry`, consumers of the list and show views need to surface this state. Without filtering, merged entries clutter the roadmap alongside their targets, creating confusion about which entry is canonical.

## Scope

### In scope

1. **Data model** -- Add `merged_into: Option<String>` and `merged_from: Vec<String>` to `PonderEntry` in `ponder.rs` (serde-compatible, backward-compatible defaults).
2. **CLI `sdlc ponder list`** -- Hide entries with `merged_into` set by default. Add `--all` flag to include them, displaying status as `parked -> <target>` indicator.
3. **CLI `sdlc ponder show`** -- When `merged_into` is set, print a redirect banner before the normal output: `This entry was merged into '<target>'. Use 'sdlc ponder show <target>' instead.`
4. **REST `GET /api/roadmap`** -- Include `merged_into` and `merged_from` fields in each entry's JSON. Filter out merged entries by default; accept `?all=true` query parameter to include them.
5. **REST `GET /api/roadmap/:slug`** -- Include `merged_into` and `merged_from` in the response JSON. When `merged_into` is set, include a `redirect_banner` string field.
6. **Frontend types** -- Add `merged_into` and `merged_from` fields to `PonderSummary` and `PonderDetail` TypeScript interfaces.
7. **Frontend list (PonderPage)** -- Filter out merged entries in the default view. Provide a toggle or "Show merged" option. Merged entries display a visual indicator (e.g., arrow icon and target slug).
8. **Frontend detail** -- When viewing a merged entry, display a prominent redirect banner with a link to the target entry.

### Out of scope

- The `sdlc ponder merge` command itself (that is `ponder-merge-cli`)
- Undo/unmerge functionality
- Multi-hop redirect chains (if A merged into B and B merged into C)

## Data Model Changes

### PonderEntry (crates/sdlc-core/src/ponder.rs)

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub merged_into: Option<String>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub merged_from: Vec<String>,
```

Both use `serde(default)` for backward compatibility with existing YAML files that lack these fields.

## CLI Changes

### `sdlc ponder list`

- Add `--all` flag to `PonderSubcommand::List`
- Default behavior: `entries.retain(|e| e.merged_into.is_none())`
- When `--all`: show all entries; for entries with `merged_into`, display status column as `parked -> <target>`
- JSON output: include `merged_into` field on all entries (even when `--all` is not set, JSON always includes all entries for programmatic consumers)

### `sdlc ponder show <slug>`

- After loading the entry, if `merged_into` is Some, print:
  ```
  Note: This entry was merged into '<target>'. Use `sdlc ponder show <target>` instead.
  ```
- Then show the entry details as normal
- JSON output: include `merged_into` and `merged_from` fields

## REST API Changes

### `GET /api/roadmap`

- Accept optional query param `all` (boolean, default false)
- When `all=false` (default): filter out entries where `merged_into` is set
- Each entry JSON object includes: `"merged_into": null | "<slug>"`, `"merged_from": []`

### `GET /api/roadmap/:slug`

- Include `merged_into` and `merged_from` fields in response
- Include `redirect_banner: string | null` -- populated when `merged_into` is set

## Frontend Changes

### Types (frontend/src/lib/types.ts)

```typescript
export interface PonderSummary {
  // ...existing fields...
  merged_into: string | null
  merged_from: string[]
}

export interface PonderDetail {
  // ...existing fields...
  merged_into: string | null
  merged_from: string[]
  redirect_banner: string | null
}
```

### PonderPage list view

- Default: filter `entries.filter(e => !e.merged_into)` on client side (server also filters, but defense in depth)
- Add a small toggle: "Show merged" checkbox or icon button
- Merged entries in list: show a muted row with right-arrow icon and "merged into <target>" text, linking to the target

### Ponder detail view

- When `merged_into` is present, render an info/warning banner at the top:
  - "This entry was merged into [target-link]. Click to view the canonical entry."
  - Styled as an info banner (blue/gray), not an error

## Error Cases

None specific to this feature -- all error handling is in the merge command itself.

## Testing

- Unit test: PonderEntry with `merged_into` / `merged_from` serde roundtrip
- Unit test: `PonderEntry::list` returns all entries (filtering is a consumer concern)
- CLI integration: `sdlc ponder list` hides merged entries; `--all` shows them
- CLI integration: `sdlc ponder show` on merged entry prints redirect banner
- REST: `GET /api/roadmap` filters merged entries; `?all=true` includes them
- REST: `GET /api/roadmap/:slug` includes `merged_into` in response
