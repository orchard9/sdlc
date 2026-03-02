# Spec: sdlc backlog CLI commands

## Overview

Add `sdlc backlog` as a top-level command group in the CLI. The data layer (`sdlc-core/src/backlog.rs`) is already complete with `BacklogStore`, `BacklogItem`, `BacklogKind`, and `BacklogStatus` types. This feature implements only the CLI presentation layer in `crates/sdlc-cli/src/cmd/backlog.rs`, wiring it into `cmd/mod.rs` and `main.rs`.

## Background

The backlog is a project-level parking lot for cross-feature concerns discovered during autonomous agent runs. Agents call `sdlc backlog add` at the moment of discovery — not deferred to session end — capturing concerns, ideas, or debt items that have no other natural home. Items persist in `.sdlc/backlog.yaml` with sequential B-prefixed IDs (B1, B2, …).

## Subcommands

### `sdlc backlog add <title...> [options]`

Captures a new backlog item. The title is variadic (multiple words joined without quoting).

Options:
- `--description <text>` — multi-line context string
- `--kind <concern|idea|debt>` — default: `concern`
- `--evidence <text>` — file path, function name, or failing test reference
- `--source-feature <slug>` — feature that was active when discovered; if omitted, auto-infer from `active_features` in `.sdlc/state.yaml` (use the last entry); print a warning if not found, but never block the add

Confirmation output (required format for agent transcript auditability):
```
Backlog item <ID> recorded: "<title>" [<source_feature>]
```
If source_feature is None, print `[none]` in brackets.

### `sdlc backlog list [options]`

List backlog items. Default: open items only (status=open).

Options:
- `--all` — show all statuses (open, parked, promoted)
- `--status <open|parked|promoted>` — filter by specific status (mutually exclusive with `--all`)
- `--source-feature <slug>` — filter by feature origin

Table columns: `ID | KIND | STATUS | SOURCE | TITLE`

### `sdlc backlog park <id> --reason <reason>`

Park an item. The `--reason` flag is required; reject without it:
```
error: Parking requires a reason (--reason). Example: --reason "revisit after v14"
```
The core layer already enforces a non-empty reason.

Confirmation output:
```
Parked B1: <reason>
```

### `sdlc backlog promote <id> [--slug <feature-slug>] [--milestone <milestone-slug>]`

Promote an item to a tracked feature.

Behavior:
1. Derive the feature slug from `--slug` if provided, or auto-generate from the item title (kebab-case, max 40 chars, truncate at word boundary)
2. Call `sdlc feature create <slug> --title "<title>" --description "<description>"` internally (via `Feature::create_with_description` from sdlc-core)
3. Update state: add to `active_features` in state.yaml
4. Mark the backlog item as promoted via `BacklogStore::mark_promoted`
5. If `--milestone <slug>` provided: call `Milestone::load` + `milestone.add_feature(<feature_slug>)` + `milestone.save` to link the feature
6. Output:
```
Promoted B1 → feature: <feature-slug>
```
If milestone was provided, also print:
```
Added to milestone: <milestone-slug>
```

### `sdlc backlog show <id>`

Show full details for a single item.

Fields shown:
- ID, Kind, Status
- Title
- Description (if set)
- Evidence (if set)
- Source Feature (if set)
- Park Reason (if set, status=parked)
- Promoted To (if set, status=promoted)
- Created At, Updated At

## JSON output (`--json` / `-j` global flag)

All subcommands support `--json`. Output is a JSON object representing the item(s):
- `add`, `park`, `promote`, `show`: single item object
- `list`: JSON array of item objects

Item JSON shape matches `BacklogItem` serde output (snake_case fields).

## Wire-up

1. Create `crates/sdlc-cli/src/cmd/backlog.rs`
2. Add `pub mod backlog;` to `crates/sdlc-cli/src/cmd/mod.rs`
3. Add to `main.rs`:
   - Import `cmd::backlog::BacklogSubcommand`
   - Add `Backlog` variant to `Commands` enum:
     ```rust
     /// Manage the project-level backlog (cross-feature concerns, ideas, debt)
     Backlog {
         #[command(subcommand)]
         subcommand: BacklogSubcommand,
     },
     ```
   - Add dispatch in `main()`:
     ```rust
     Commands::Backlog { subcommand } => cmd::backlog::run(&root, subcommand, cli.json),
     ```

## Error handling

- Unknown item ID: propagate `SdlcError::BacklogItemNotFound` — the core already returns this
- Park without reason: `--reason` is a required argument in Clap; error message as shown above
- Promote with `--milestone` pointing to nonexistent milestone: propagate the error from `Milestone::load`
- State file absent: propagate; do not create state — backlog commands require an initialized project

## Source feature auto-inference

When `--source-feature` is omitted on `add`:
1. Load `State` from `.sdlc/state.yaml`
2. Take `state.active_features.last()` if non-empty
3. If found, use it silently (no output)
4. If `active_features` is empty, use `None` and print to stderr:
   ```
   warning: no active feature found in state.yaml; source_feature not recorded
   ```

## Non-goals

- REST API routes (not in scope)
- UI changes (not in scope)
- Editing existing backlog items (not in scope)
- Pagination of list output (not in scope)
