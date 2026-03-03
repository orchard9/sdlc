# QA Plan: Finding-Closure Protocol — sdlc-next Template + CLAUDE.md Ethos

## Scope

This is a documentation change to two text files. QA verifies that the exact required
content is present at the correct location in each file, and that no existing content
was inadvertently modified or removed.

## Test Cases

### TC-1: SDLC_NEXT_COMMAND has a dedicated review/audit approval section

**How:** Read `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`.

**Pass criteria:**
- The `SDLC_NEXT_COMMAND` constant has a separate approval block for `approve_review` and `approve_audit`
- That block mentions all three dispositions: "Fix now", "Track", "Accept"
- That block prohibits silent skips (e.g., "No finding may be silently skipped")
- That block distinguishes targeted fixes from `fix-all` / `remediate`

**Fail criteria:** Any of the above elements are absent.

### TC-2: SDLC_NEXT_PLAYBOOK has the review/audit protocol step

**How:** Read `SDLC_NEXT_PLAYBOOK` in the same file.

**Pass criteria:**
- There is a step (step 5a or equivalent) for `approve_review` and `approve_audit`
- It references fix now / track / accept
- It prohibits silent skips

**Fail criteria:** The playbook only covers generic approval without the three-disposition protocol.

### TC-3: CLAUDE.md has the ethos bullet

**How:** Read `CLAUDE.md`.

**Pass criteria:**
- The Ethos section contains "**Audits and reviews close every finding.**"
- The bullet includes the three actions (fix / track / accept)
- It distinguishes `fix-all` / `remediate` from targeted fixes

**Fail criteria:** The bullet is absent or incomplete.

### TC-4: Build passes with no regressions

**How:** Run `SDLC_NO_NPM=1 cargo test --all`.

**Pass criteria:** All tests pass, zero compilation errors.

**Fail criteria:** Any test fails or compilation errors occur.

## QA Environment

- Local development environment
- No external dependencies required (documentation-only change)
