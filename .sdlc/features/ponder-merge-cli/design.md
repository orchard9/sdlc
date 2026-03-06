# Design: sdlc ponder merge — CLI command and core data model

## Architecture

This feature touches two crates:
1. **sdlc-core** (`ponder.rs`) — data model additions + `merge_entries()` pure function
2. **sdlc-cli** (`cmd/ponder.rs`) — `Merge` subcommand variant + `merge()` handler

No server routes, no frontend changes, no new crates.

## Data Model (sdlc-core/src/ponder.rs)

### PonderEntry additions

```rust
// Add to PonderEntry struct:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub merged_into: Option<String>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub merged_from: Vec<String>,
```

Both use `serde(default)` — existing YAML files without these fields deserialize cleanly.

### New error variant (error.rs)

```rust
#[error("cannot merge ponder entry: {0}")]
PonderMergeError(String),
```

### Core merge function

```rust
/// Merge source ponder entry into target. Returns counts of copied items.
pub fn merge_entries(root: &Path, source_slug: &str, target_slug: &str) -> Result<MergeResult>
```

Located in `ponder.rs`. The function is a single transaction-like sequence:

```
validate_preconditions(source, target)
  → copy_sessions(source_dir, target_dir) → returns session_count
  → copy_artifacts(source_dir, target_dir) → returns artifact_count  
  → merge_team(source_slug, target_slug) → returns member_count
  → update_target_manifest(target, source)
  → park_source(source, target_slug)
  → MergeResult { sessions_copied, artifacts_copied, team_members_copied }
```

### MergeResult struct

```rust
pub struct MergeResult {
    pub sessions_copied: u32,
    pub artifacts_copied: u32,
    pub team_members_copied: u32,
}
```

## Detailed Steps

### 1. Validate preconditions

```rust
fn validate_merge_preconditions(source: &PonderEntry, target: &PonderEntry) -> Result<()>
```

Checks:
- source.slug != target.slug (self-merge)
- source.status != Committed
- target.status != Committed  
- source.merged_into.is_none() (already merged)

All errors use `SdlcError::PonderMergeError(message)`.

### 2. Copy sessions

Read all files from `source_dir/sessions/session-NNN.md`. For each:
- Read content
- Prepend: `<!-- merged from: {source_slug}, original session: {N} -->\n`
- Call `workspace::write_session(target_dir, &modified_content)` to get next target number
- Count total copied

### 3. Copy artifacts

List files in source dir, skip: `manifest.yaml`, `team.yaml`, `sessions/` (dir).
For each file:
- If target has a file with same name: copy as `{source_slug}--{filename}`
- Otherwise: copy with original name
- Use `std::fs::read` + `crate::io::atomic_write` for binary safety

### 4. Merge team

Load both teams. For each source partner:
- If target team has a member with same `name`, skip
- Otherwise, add to target team
- Save target team

### 5. Update target manifest

```rust
target.merged_from.push(source_slug.to_string());
// Union tags
for tag in &source.tags {
    target.add_tag(tag);
}
// Bump session count
target.sessions += sessions_copied;
target.updated_at = Utc::now();
target.save(root)?;
```

### 6. Park source

```rust
source.status = PonderStatus::Parked;
source.merged_into = Some(target_slug.to_string());
source.updated_at = Utc::now();
source.save(root)?;
```

## CLI Design (sdlc-cli/src/cmd/ponder.rs)

### New subcommand variant

```rust
/// Merge a ponder entry into another
Merge {
    /// Source entry to merge (will be parked)
    source: String,
    /// Target entry to merge into
    #[arg(long)]
    into: String,
},
```

### Handler

```rust
fn merge(root: &Path, source: &str, target: &str, json: bool) -> anyhow::Result<()> {
    let result = sdlc_core::ponder::merge_entries(root, source, target)
        .with_context(|| format!("failed to merge '{source}' into '{target}'"))?;

    // Remove source from active_ponders
    if let Ok(mut state) = State::load(root) {
        state.remove_ponder(source);
        state.save(root)?;
    }

    // Output
    if json { ... } else {
        println!("Merged '{}' into '{}': {} sessions, {} artifacts, {} team members copied",
            source, target,
            result.sessions_copied, result.artifacts_copied, result.team_members_copied);
    }
}
```

### List changes

Add `--all` flag to `PonderSubcommand::List`:

```rust
List {
    #[arg(long)]
    status: Option<String>,
    /// Show all entries including merged redirects
    #[arg(long)]
    all: bool,
},
```

Default behavior: filter out entries where `merged_into.is_some()`.
With `--all`: show all, display status column as `parked -> {target}` for merged entries.

### Show changes

In the `show()` function, after loading the entry, if `merged_into` is Some:

```rust
if let Some(ref target) = entry.merged_into {
    if !json {
        println!("⚠ This entry was merged into '{}'. Use `sdlc ponder show {}` instead.\n", target, target);
    }
}
```

In JSON output, include `merged_into` and `merged_from` fields.

## File Layout

No new files created. Changes to existing files:

| File | Change |
|---|---|
| `crates/sdlc-core/src/ponder.rs` | Add `merged_into`, `merged_from` fields; `MergeResult` struct; `merge_entries()` fn; `validate_merge_preconditions()` fn; tests |
| `crates/sdlc-core/src/error.rs` | Add `PonderMergeError(String)` variant |
| `crates/sdlc-cli/src/cmd/ponder.rs` | Add `Merge` variant; `merge()` handler; update `list()` for `--all`; update `show()` for redirect banner |

## Error handling

All errors use `?` propagation. No `unwrap()` in library code. The merge function does not attempt rollback — if a step fails partway through, the state is partially merged. This is acceptable because:
- Sessions are append-only (copied sessions don't harm target)
- Artifacts are additive (extra files don't harm target)
- The source is only parked as the final step, so a mid-failure leaves source intact
