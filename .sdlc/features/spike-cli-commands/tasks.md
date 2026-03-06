# Tasks: Spike CLI — list, show, promote subcommands

## T1 — Add spike.rs with `spike list` subcommand

File: `crates/sdlc-cli/src/cmd/spike.rs`

- Define `SpikeSubcommand` enum with `List`, `Show { slug }`, `Promote { slug, as_slug }`
- Implement `fn list(root, json)`: call `sdlc_core::spikes::list`, output table with
  columns SLUG | VERDICT | DATE | TITLE using `print_table`, or `[]`/`"No spikes."` if empty
- JSON output: array of `{ slug, title, verdict, date, ponder_slug, knowledge_slug }`

## T2 — Add `spike show <slug>` subcommand

In the same `spike.rs` file:

- Implement `fn show(root, slug, json)`: call `sdlc_core::spikes::load`
- Human output: slug/title, verdict + date, the_question, full findings.md content
- ADOPT hint: "Hint: ADOPT — consider /sdlc-hypothetical-planning to implement this technology."
- REJECT hint: "Hint: REJECT — findings stored in knowledge base as '<knowledge_slug>'." (if set)
- ADAPT with ponder_slug: "Ponder: already promoted → '<ponder_slug>'"
- JSON output: `{ slug, title, verdict, date, the_question, ponder_slug, knowledge_slug, findings_content }`

## T3 — Add `spike promote <slug> [--as <ponder-slug>]` subcommand

In the same `spike.rs` file:

- Implement `fn promote(root, slug, as_slug, json)`: call `sdlc_core::spikes::promote_to_ponder`
- Human output: "Promoted spike '<slug>' to ponder '<ponder_slug>'.\nNext: sdlc ponder show <ponder_slug>"
- JSON output: `{ spike_slug, ponder_slug }`
- Add `pub fn run(root, subcmd, json)` dispatch function

## T4 — Register `spike` subcommand in main.rs and mod.rs

- `crates/sdlc-cli/src/cmd/mod.rs`: add `pub mod spike;`
- `crates/sdlc-cli/src/main.rs`:
  - Import `cmd::spike::SpikeSubcommand`
  - Add `Commands::Spike { #[command(subcommand)] subcommand: SpikeSubcommand }` with help
    "Manage spike findings (list, show, promote)"
  - Add dispatch arm `Commands::Spike { subcommand } => cmd::spike::run(&root, subcommand, cli.json)`
