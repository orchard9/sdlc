# QA Plan: Product Summary format contract in /sdlc-ponder skill

## Scope

This QA plan covers the changes to `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`. The change is documentation-only (skill instruction text). Testing focuses on correctness of the text content, build integrity, and propagation behavior.

## Test Cases

### TC-1: Build succeeds

**Action:** `SDLC_NO_NPM=1 cargo build --all`
**Expected:** Zero compiler errors. The `const &str` constants are valid Rust string literals.

### TC-2: Tests pass

**Action:** `SDLC_NO_NPM=1 cargo test --all`
**Expected:** All tests pass. No regressions from the string constant changes.

### TC-3: Clippy passes

**Action:** `cargo clippy --all -- -D warnings`
**Expected:** No warnings.

### TC-4: SDLC_PONDER_COMMAND contains Product Summary schema

**Action:** Read `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`
**Expected:**
- Contains `## Product Summary` heading
- Contains all four H3 subsections: `### What we explored`, `### Key shifts`, `### Implications`, `### Still open`
- Contains the statement that H3 labels are "stable and locked"
- Contains the rule that Implications must use product language, not tech jargon
- Contains the rule that Still open items are phrased as decisions, not technical tasks
- The Product Summary schema appears in the Session Log Protocol section (before "The only correct logging procedure")

### TC-5: SDLC_PONDER_COMMAND Ending the session references Product Summary

**Action:** Read `SDLC_PONDER_COMMAND` constant
**Expected:** The `## Ending the session` section step 1 references the Product Summary requirement so the agent knows to include it.

### TC-6: SDLC_PONDER_PLAYBOOK mentions Product Summary

**Action:** Read `SDLC_PONDER_PLAYBOOK` constant
**Expected:** Step 6 mentions `## Product Summary` and the four H3s (concise form appropriate for the playbook).

### TC-7: SDLC_PONDER_SKILL mentions Product Summary

**Action:** Read `SDLC_PONDER_SKILL` constant
**Expected:** Step 6 references the `## Product Summary` section requirement with the locked subsection labels.

### TC-8: sdlc update installs updated content

**Action:** Run `sdlc update` (or inspect the installed `~/.claude/commands/sdlc-ponder.md`)
**Expected:** The installed file contains the `## Product Summary` section with all four H3 subsections.

## Pass Criteria

All 8 test cases pass. No regressions in cargo test output. The Product Summary schema is present, complete, and consistent across all three skill variants (COMMAND, PLAYBOOK, SKILL).
