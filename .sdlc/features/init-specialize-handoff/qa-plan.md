# QA Plan: Init Phase 6 → Specialize Handoff

## Scope

This is a template-text-only change to 3 Rust string constants. No logic, no state machine, no runtime behavior changes.

## Test Strategy

### 1. Compilation Check
- `SDLC_NO_NPM=1 cargo test --all` — confirms all constants are valid Rust string literals and no tests broke
- `cargo clippy --all -- -D warnings` — no new warnings

### 2. Content Verification
Manually verify in the edited file:
- [ ] `SDLC_INIT_COMMAND` Phase 6 references `/sdlc-specialize` workflow (6 steps: survey, summarize, design, gate, generate, update)
- [ ] `SDLC_INIT_COMMAND` Phase 7 (Seed First Milestone) is unchanged and correctly numbered
- [ ] `SDLC_INIT_COMMAND` Finish block references specialize output
- [ ] `SDLC_INIT_PLAYBOOK` steps renumbered correctly (specialize=8, seed=9, finish=10)
- [ ] `SDLC_INIT_SKILL` steps renumbered correctly (specialize=8, seed=9)
- [ ] No remnants of old Phase 6 sub-phases (6a, 6b, 6c, 6d) in any template
- [ ] No references to inline roster design, agent file generation patterns, or AGENTS.md update instructions remain

### 3. Negative Check
- [ ] `sdlc_specialize.rs` is untouched
- [ ] No other files modified beyond `sdlc_init.rs`

## Pass Criteria

All compilation succeeds, all content checks pass, old Phase 6 fully removed.
