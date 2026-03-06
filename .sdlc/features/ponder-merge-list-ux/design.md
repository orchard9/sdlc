# Design: ponder list/show UX for merged entries

## Architecture Overview

This feature touches four layers: data model, CLI, REST API, and frontend. Changes flow bottom-up: data model fields enable CLI/REST filtering, which the frontend consumes.

## Layer 1: Data Model (ponder.rs)

Add two fields to `PonderEntry`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub merged_into: Option<String>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub merged_from: Vec<String>,
```

No new methods needed -- the merge command (separate feature) sets these. This feature only reads them.

## Layer 2: CLI (cmd/ponder.rs)

### List subcommand

Add `--all` flag:

```
List {
    #[arg(long)]
    status: Option<String>,
    #[arg(long)]
    all: bool,
}
```

In `fn list()`:
- After loading and status-filtering, if `!all`, add: `entries.retain(|e| e.merged_into.is_none())`
- In table output, for entries with `merged_into`, display status as `parked -> <target>`
- In JSON output, always include `merged_into` field

### Show subcommand

In `fn show()`:
- After loading entry, before any output, if `entry.merged_into.is_some()`:
  - Human: print `Note: This entry was merged into '<target>'. Use 'sdlc ponder show <target>' instead.` followed by blank line
  - JSON: include `"merged_into"` and `"merged_from"` fields in the output object

## Layer 3: REST API (routes/roadmap.rs)

### GET /api/roadmap

Add query parameter struct:

```rust
#[derive(serde::Deserialize)]
pub struct ListPondersQuery {
    #[serde(default)]
    pub all: Option<bool>,
}
```

In `list_ponders`:
- Accept `Query(params): Query<ListPondersQuery>`
- After building the list, if `!params.all.unwrap_or(false)`, filter out entries with `merged_into`
- Add `"merged_into"` and `"merged_from"` to each entry's JSON

### GET /api/roadmap/:slug

- Add `"merged_into"` and `"merged_from"` to the response JSON
- Add `"redirect_banner"`: if `merged_into` is Some, set to `"This entry was merged into '<target>'"`, else null

## Layer 4: Frontend

### Types (types.ts)

Add to both `PonderSummary` and `PonderDetail`:

```typescript
merged_into: string | null
merged_from: string[]
```

Add to `PonderDetail` only:

```typescript
redirect_banner: string | null
```

### API client (client.ts)

Update `getRoadmap` to accept optional `all` param:

```typescript
getRoadmap: (all?: boolean) =>
  request<PonderSummary[]>(`/api/roadmap${all ? '?all=true' : ''}`),
```

### PonderPage list view

- Client-side filter as defense-in-depth: `entries.filter(e => !e.merged_into)`
- Add a small toggle button: "Show all" that passes `all=true` to the API
- Merged entries render with reduced opacity, an arrow icon, and text "-> <target>" linking to the target

### Ponder detail view

- When `merged_into` is set, render an info banner at the top of the detail panel:
  - Blue/gray info style (not error/warning)
  - Text: "This entry was merged into [target]" with a clickable link
  - Banner does not block viewing the rest of the entry content

## Data Flow

```
PonderEntry.merged_into (YAML)
  -> CLI list: filter if not --all, format status column
  -> CLI show: print banner
  -> REST list: filter if not ?all=true, include in JSON
  -> REST show: include in JSON + redirect_banner
  -> Frontend list: client filter + toggle
  -> Frontend detail: info banner
```

## Backward Compatibility

- `serde(default)` on both fields means existing YAML without these fields deserializes correctly (None / empty Vec)
- REST responses gain new fields -- additive, non-breaking for API consumers
- CLI gains `--all` flag -- additive, default behavior unchanged for existing scripts
