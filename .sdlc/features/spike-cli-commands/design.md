# Design: Spike CLI — list, show, promote subcommands

## Architecture

This is a pure CLI feature. No server routes, no UI, no new data types.

```
crates/sdlc-cli/src/cmd/spike.rs   ← new file (this feature)
crates/sdlc-cli/src/cmd/mod.rs     ← add `pub mod spike;`
crates/sdlc-cli/src/main.rs        ← add Commands::Spike + dispatch arm
crates/sdlc-core/src/spikes.rs     ← existing, read-only (data layer)
```

## Module Structure: spike.rs

```rust
pub enum SpikeSubcommand {
    List,
    Show { slug: String },
    Promote {
        slug: String,
        #[arg(long = "as")]
        as_slug: Option<String>,
    },
}

pub fn run(root: &Path, subcmd: SpikeSubcommand, json: bool) -> anyhow::Result<()>
```

Three handler functions:
- `fn list(root, json)` — calls `spikes::list`, prints table or JSON array
- `fn show(root, slug, json)` — calls `spikes::load`, prints full details + hints
- `fn promote(root, slug, as_slug, json)` — calls `spikes::promote_to_ponder`

## Table Output (list)

```
SLUG             VERDICT  DATE        TITLE
---------------  -------  ----------  -------------------------
my-spike         ADOPT    2026-03-04  Can we use X for Y?
old-spike        REJECT   2026-01-15  Worth switching to Z?
```

Uses `print_table` from `crate::output`.

## Show Output

```
Spike: my-spike — Can we use X for Y?
Verdict: ADOPT    Date: 2026-03-04
The Question: Can we use X for Y?

--- Findings ---
<raw findings.md content>

Hint: ADOPT — consider /sdlc-hypothetical-planning to implement this technology.
```

For REJECT with knowledge_slug:
```
Hint: REJECT — findings stored in knowledge base as 'spike-my-spike'.
```

For ADAPT with ponder_slug already set:
```
Ponder: already promoted → 'my-spike'
```

## Promote Output

```
Promoted spike 'my-spike' to ponder 'my-spike'.
Next: sdlc ponder show my-spike
```

## JSON Shapes

### list --json
```json
[
  {
    "slug": "my-spike",
    "title": "Can we use X for Y?",
    "verdict": "ADOPT",
    "date": "2026-03-04",
    "ponder_slug": null,
    "knowledge_slug": null
  }
]
```

### show --json
```json
{
  "slug": "my-spike",
  "title": "Can we use X for Y?",
  "verdict": "ADOPT",
  "date": "2026-03-04",
  "the_question": "Can we use X for Y?",
  "ponder_slug": null,
  "knowledge_slug": null,
  "findings_content": "# Spike: ..."
}
```

### promote --json
```json
{
  "spike_slug": "my-spike",
  "ponder_slug": "my-spike"
}
```

## Error Handling

- Unknown spike slug → propagate `SdlcError` from core via `anyhow::Context`
- No spikes directory / empty → print "No spikes." (human) or `[]` (JSON), exit 0
- `promote` on ADOPT/REJECT spikes → core does not block it; the CLI does not add an
  extra guard (the user may have a valid reason; the ponder will be seeded with findings)

## Dependencies

No new crate dependencies needed. Uses:
- `sdlc_core::spikes` (already in workspace)
- `crate::output::{print_json, print_table}` (already in sdlc-cli)
- `anyhow`, `clap` (already in sdlc-cli)
