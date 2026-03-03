# Tasks: Smarter sdlc-init Finish

## T1: Add Phase 7 (Seed First Milestone) to SDLC_INIT_COMMAND

Update the `SDLC_INIT_COMMAND` constant in `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`.

- Remove the existing "Finish" section's generic "Next: /sdlc-ponder or /sdlc-plan" handoff
- Add Phase 7 section with three sub-phases:
  - **7a: Assess scope thickness** — criteria for thin vs. thick scope
  - **7b: Synthesize milestone structure** — derivation rules for slug, title, vision, features, acceptance test
  - **7c: Seed via CLI** — inline sdlc-plan execution steps using `sdlc milestone create/update/set-acceptance-test` and `sdlc feature create` / `sdlc milestone add-feature`
- Update the Finish summary block to include the seeded milestone line
- Update the final `**Next:**` to point at `/sdlc-prepare <slug>` (or `/sdlc-ponder` if scope thin)

## T2: Update SDLC_INIT_PLAYBOOK for Phase 7

Update the `SDLC_INIT_PLAYBOOK` constant (Gemini/OpenCode variant) in the same file.

- Add step 11: Seed first milestone (assess → synthesize → create)
- Update the final **Next** to include milestone-focused path

## T3: Update SDLC_INIT_SKILL for Phase 7

Update the `SDLC_INIT_SKILL` constant (generic agents variant) in the same file.

- Add workflow step for seeding
- Update the Outcome table to include:
  - Scope thick → milestone seeded → `**Next:** /sdlc-prepare <slug>`
  - Scope thin → no seed → `**Next:** /sdlc-ponder`

## T4: Run `cargo build` to verify no compilation errors

Since this is text-only (string constant changes), compilation should pass immediately. Verify with:

```bash
SDLC_NO_NPM=1 cargo build -p sdlc-cli 2>&1
```

## T5: Run `sdlc update` to reinstall templates

After the build succeeds, reinstall the updated templates to user directories:

```bash
cargo run -p sdlc-cli -- update
```

Verify the updated `sdlc-init` command is installed in `~/.claude/commands/sdlc-init.md`.
