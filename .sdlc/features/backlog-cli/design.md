# Design: sdlc backlog CLI commands

## Summary

This feature adds the `sdlc backlog` command group as a thin CLI presentation layer over the existing `sdlc-core::backlog` data layer. No new data structures or persistence logic is needed — only the CLI module, enum, dispatch, and wire-up into `main.rs`.

## Module Layout

```
crates/sdlc-cli/src/cmd/
  backlog.rs          ← new file (this feature)
  mod.rs              ← add: pub mod backlog;
crates/sdlc-cli/src/
  main.rs             ← add: Backlog variant + dispatch
```

## backlog.rs Structure

```rust
use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    backlog::{BacklogKind, BacklogStatus, BacklogStore},
    feature::Feature,
    milestone::Milestone,
    state::State,
};
use std::path::Path;

#[derive(Subcommand)]
pub enum BacklogSubcommand {
    Add { ... },
    List { ... },
    Park { ... },
    Promote { ... },
    Show { ... },
}

pub fn run(root: &Path, subcmd: BacklogSubcommand, json: bool) -> anyhow::Result<()> { ... }
```

## Subcommand Designs

### Add

```rust
Add {
    #[arg(required = true)]
    title: Vec<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long, default_value = "concern", value_parser = parse_kind)]
    kind: BacklogKind,
    #[arg(long)]
    evidence: Option<String>,
    #[arg(long)]
    source_feature: Option<String>,
}
```

Implementation steps:
1. Join `title` vec with spaces
2. If `source_feature` is `None`, attempt auto-inference:
   - Load `State::load(root)` — if ok, take `state.active_features.last().cloned()`
   - If empty, print warning to stderr; leave as `None`
3. Call `BacklogStore::add(root, title, kind, description, evidence, source_feature)`
4. Print: `Backlog item {id} recorded: "{title}" [{source}]` where source is `source_feature.as_deref().unwrap_or("none")`

### List

```rust
List {
    #[arg(long)]
    all: bool,
    #[arg(long, conflicts_with = "all")]
    status: Option<String>,
    #[arg(long)]
    source_feature: Option<String>,
}
```

Implementation steps:
1. Resolve status filter:
   - `--all` → `None` (no filter)
   - `--status <s>` → parse to `BacklogStatus`
   - default (neither) → `Some(BacklogStatus::Open)`
2. Call `BacklogStore::list(root, status_filter, source_feature.as_deref())`
3. Print table: `ID | KIND | STATUS | SOURCE | TITLE`
   - SOURCE column: `source_feature.as_deref().unwrap_or("-")`

### Park

```rust
Park {
    id: String,
    #[arg(long, required = true)]
    reason: Vec<String>,
}
```

Using `Vec<String>` for reason allows multi-word values without quoting. Join with spaces.

Implementation:
1. Call `BacklogStore::park(root, &id, reason)`
2. Print: `Parked {id}: {reason}`

### Promote

```rust
Promote {
    id: String,
    #[arg(long)]
    slug: Option<String>,
    #[arg(long)]
    milestone: Option<String>,
}
```

Implementation:
1. Load item via `BacklogStore::get(root, &id)`
2. Derive feature slug: `--slug` if provided, else `slugify(&item.title)` (kebab-case, lowercase, strip non-alphanum, max 40 chars, trim trailing dashes)
3. Create feature: `Feature::create_with_description(root, &feature_slug, &item.title, item.description.clone())`
4. Update state: `State::load` → `state.add_active_feature(&feature_slug)` → `state.save`
5. Mark promoted: `BacklogStore::mark_promoted(root, &id, &feature_slug)`
6. If `--milestone <ms>`: `Milestone::load(root, &ms)` → `milestone.add_feature(&feature_slug)` → `milestone.save`
7. Print:
   ```
   Promoted {id} → feature: {feature_slug}
   [Added to milestone: {ms}]  ← only if --milestone provided
   ```

### Show

```rust
Show {
    id: String,
}
```

Print all fields. Use `println!` for each non-None field. For JSON, serialize item directly.

## Slug Generation for Promote

```rust
fn slugify(title: &str) -> String {
    let s: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse runs of dashes, trim leading/trailing
    let s = s.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    // Truncate at word boundary up to 40 chars
    if s.len() <= 40 {
        s
    } else {
        let truncated = &s[..40];
        match truncated.rfind('-') {
            Some(pos) => s[..pos].to_string(),
            None => truncated.to_string(),
        }
    }
}
```

## BacklogKind value_parser

```rust
fn parse_kind(s: &str) -> Result<BacklogKind, String> {
    match s {
        "concern" => Ok(BacklogKind::Concern),
        "idea" => Ok(BacklogKind::Idea),
        "debt" => Ok(BacklogKind::Debt),
        other => Err(format!("unknown kind '{other}'; expected concern, idea, or debt")),
    }
}
```

## BacklogStatus parsing (for --status)

```rust
fn parse_status(s: &str) -> anyhow::Result<BacklogStatus> {
    match s {
        "open" => Ok(BacklogStatus::Open),
        "parked" => Ok(BacklogStatus::Parked),
        "promoted" => Ok(BacklogStatus::Promoted),
        other => anyhow::bail!("unknown status '{other}'; expected open, parked, or promoted"),
    }
}
```

## JSON output

All subcommands check `json: bool`. When true:
- `add`, `park`, `promote`, `show` → `print_json(&item)?`
- `list` → `print_json(&items)?`

`BacklogItem` derives `Serialize` in sdlc-core, so this works directly.

## Wire-up in main.rs

```rust
// In imports:
use cmd::backlog::BacklogSubcommand;

// In Commands enum:
/// Manage the project-level backlog (cross-feature concerns, ideas, debt)
Backlog {
    #[command(subcommand)]
    subcommand: BacklogSubcommand,
},

// In match cli.command:
Commands::Backlog { subcommand } => cmd::backlog::run(&root, subcommand, cli.json),
```

## Testing approach

Integration tests in `crates/sdlc-cli/tests/integration.rs` will verify:
- `sdlc backlog add` produces the required confirmation string
- `sdlc backlog list` defaults to open items only
- `sdlc backlog park <id> --reason ...` rejects without reason
- `sdlc backlog promote <id>` creates a feature and updates state

Unit tests in `backlog.rs` for `slugify()`.

## Dependencies

All required dependencies are already in `Cargo.toml`:
- `clap` (with `derive`) — already used across all cmd modules
- `anyhow` — already used
- `sdlc-core` — already declared as a workspace dependency in sdlc-cli

No new dependencies needed.
