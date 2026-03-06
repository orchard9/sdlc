# Spec: Spike CLI — list, show, promote subcommands

## Overview

Add `sdlc spike list | show | promote` CLI subcommands in `crates/sdlc-cli/src/cmd/spike.rs`
that wrap the already-implemented `sdlc_core::spikes` data layer. The `sdlc spike` command
makes spike findings discoverable and actionable directly from the terminal.

## Goals

- Make time-boxed spike results visible via `sdlc spike list` (tabular overview)
- Allow reading full findings via `sdlc spike show <slug>`
- Allow promoting an ADAPT spike to a ponder entry via `sdlc spike promote <slug>`
- Follow the existing CLI patterns (investigate.rs / ponder.rs) exactly

## Non-Goals

- No modification of findings.md (the spike data layer is read-only on findings)
- No new state fields beyond what `SpikeEntry` / `state.yaml` already captures
- No server REST routes in this feature (CLI only)

## Subcommands

### `sdlc spike list`

Prints a table with columns: `SLUG | VERDICT | DATE | TITLE`

- Calls `sdlc_core::spikes::list(root)`
- If no spikes exist, prints "No spikes." and exits cleanly
- JSON mode: array of `{ slug, title, verdict, date, ponder_slug, knowledge_slug }`
- Sort order: date descending (delegated to core)

### `sdlc spike show <slug>`

Prints full spike details including the raw `findings.md` content.

Human-readable output:
```
Spike: <slug> — <title>
Verdict: ADOPT | ADAPT | REJECT    Date: <date>
The Question: <the_question>

--- Findings ---
<raw findings.md content>
```

For ADOPT verdict: append a hint line:
```
Hint: ADOPT — consider /sdlc-hypothetical-planning to implement this technology.
```

For REJECT verdict: show `knowledge_slug` if set:
```
Hint: REJECT — findings stored in knowledge base as '<knowledge_slug>'.
```

For ADAPT verdict with `ponder_slug` set: show:
```
Ponder: already promoted → '<ponder_slug>'
```

JSON mode: full entry object plus `findings_content` field.

### `sdlc spike promote <slug> [--as <ponder-slug>]`

- Calls `sdlc_core::spikes::promote_to_ponder(root, slug, ponder_slug_override)`
- Prints the resulting ponder slug and a next-step hint:
  ```
  Promoted spike '<slug>' to ponder '<ponder-slug>'.
  Next: sdlc ponder show <ponder-slug>
  ```
- JSON mode: `{ spike_slug, ponder_slug }`
- Errors if spike not found (propagates from core)

## CLI Registration

- New file: `crates/sdlc-cli/src/cmd/spike.rs`
- Register in `crates/sdlc-cli/src/cmd/mod.rs` as `pub mod spike;`
- Register in `crates/sdlc-cli/src/main.rs` as `Commands::Spike { subcommand }` with the
  help text "Manage spike findings (list, show, promote)"

## Acceptance Criteria

1. `cargo build --all` passes with no errors or warnings
2. `SDLC_NO_NPM=1 cargo test --all` passes
3. `cargo clippy --all -- -D warnings` passes
4. `sdlc spike list` shows a table (or "No spikes." if none)
5. `sdlc spike show <slug>` prints findings and verdict hints
6. `sdlc spike promote <slug>` creates a ponder and prints slug + hint
7. `sdlc spike promote <slug> --as <ponder-slug>` uses the override slug
8. `sdlc spike list --json` returns valid JSON array
9. `sdlc spike show <slug> --json` returns valid JSON with `findings_content`
10. `sdlc spike promote <slug> --json` returns `{ spike_slug, ponder_slug }`
