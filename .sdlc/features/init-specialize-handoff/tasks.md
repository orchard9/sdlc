# Tasks: Init Phase 6 → Specialize Handoff

## T1: Replace Phase 6 in SDLC_INIT_COMMAND

Remove Phase 6 sub-phases 6a-6d (lines ~207-277) in `SDLC_INIT_COMMAND` and replace with the new "Specialize — AI Team" handoff text that instructs the agent to follow `/sdlc-specialize`.

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`

## T2: Update Finish block in SDLC_INIT_COMMAND

Update the summary checklist in the Finish section to reference specialize output instead of listing individual agent names (e.g., "Agents: via /sdlc-specialize").

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`

## T3: Replace team steps in SDLC_INIT_PLAYBOOK

Replace steps 8-10 (design team, create agents, update AGENTS.md) with a single specialize handoff step. Renumber subsequent steps (seed → 9, finish → 10).

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`

## T4: Replace team steps in SDLC_INIT_SKILL

Replace steps 8-10 with a single specialize handoff step. Renumber subsequent steps (seed → 9).

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`

## T5: Verify build and tests

Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings` to confirm no regressions.
