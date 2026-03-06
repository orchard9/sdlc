# Tasks: sdlc ponder merge — CLI command and core data model

## T1: Add data model fields and error variant

- Add `merged_into: Option<String>` and `merged_from: Vec<String>` to `PonderEntry` in `crates/sdlc-core/src/ponder.rs`
- Add `PonderMergeError(String)` variant to `SdlcError` in `crates/sdlc-core/src/error.rs`
- Add `MergeResult` struct to `ponder.rs`
- Add serde roundtrip test for new fields

## T2: Implement merge_entries core function

- Add `validate_merge_preconditions()` helper in `ponder.rs`
- Add `merge_entries(root, source_slug, target_slug) -> Result<MergeResult>` in `ponder.rs`
- Implements: session copying with merge header, artifact copying with collision prefix, team merge with dedup, target manifest update, source parking
- Add unit tests: successful merge, committed-source rejection, committed-target rejection, self-merge rejection, already-merged rejection, collision prefix test

## T3: Add CLI Merge subcommand

- Add `Merge { source, into }` variant to `PonderSubcommand` in `crates/sdlc-cli/src/cmd/ponder.rs`
- Implement `merge()` handler with state.yaml cleanup and JSON/human output
- Wire into the `run()` match

## T4: Update list and show commands

- Add `--all` flag to `PonderSubcommand::List`
- Update `list()` to filter out merged entries by default, show them with `--all`
- Update `show()` to display redirect banner when `merged_into` is set
- Include `merged_into` and `merged_from` in JSON output for both commands
