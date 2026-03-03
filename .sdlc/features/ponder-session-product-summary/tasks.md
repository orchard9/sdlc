# Tasks: Product Summary format contract in /sdlc-ponder skill

## T1: Update SDLC_PONDER_COMMAND with Product Summary schema

In `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`, update `SDLC_PONDER_COMMAND`:

1. Add a `### Product Summary section` subsection to the Session Log Protocol section, placed between the "Inline markers" subsection and "The only correct logging procedure" subsection
2. The subsection defines the four locked H3 labels (What we explored, Key shifts, Implications, Still open), the format schema, and the rules
3. Update the `## Ending the session` section step 1 to reference the Product Summary requirement

## T2: Update SDLC_PONDER_PLAYBOOK with concise Product Summary requirement

In the same file, update `SDLC_PONDER_PLAYBOOK`:

- Append to step 6 in the Steps list: mention the `## Product Summary` section with the four fixed H3s

## T3: Update SDLC_PONDER_SKILL with Product Summary requirement

In the same file, update `SDLC_PONDER_SKILL`:

- Append to step 6: reference the `## Product Summary` section requirement with locked subsection labels

## T4: Verify sdlc init/update propagation

Run `cargo build --all` and verify the constants compile. Run `SDLC_NO_NPM=1 cargo test --all` to confirm no tests break. Confirm that `sdlc update` would write the new content (smoke test via `sdlc update --dry-run` if available, otherwise verify via build output).
