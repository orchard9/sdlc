# QA Plan: Update sdlc-milestone-uat Skill with Playwright Support

## Verification Strategy

This feature is a pure skill-text change in a single Rust source file. QA focuses on:
1. **Build correctness** — the file compiles without errors
2. **Content correctness** — all three skill variants contain the required Mode A/B content
3. **Install correctness** — `sdlc update` writes the updated skill to the expected path

## Checks

### QA-1: Rust build

```bash
SDLC_NO_NPM=1 cargo build --all 2>&1 | tail -20
```

Expected: exits 0, no error output.

### QA-2: Mode A language present in COMMAND

```bash
grep -c "Mode A" crates/sdlc-cli/src/cmd/init.rs
grep -c "playwright test" crates/sdlc-cli/src/cmd/init.rs
grep -c "results.json" crates/sdlc-cli/src/cmd/init.rs
```

Expected: each grep returns ≥ 1.

### QA-3: Mode B language present in COMMAND

```bash
grep -c "Mode B" crates/sdlc-cli/src/cmd/init.rs
grep -c "getByRole\|getByTestId" crates/sdlc-cli/src/cmd/init.rs
grep -c "mcp__playwright" crates/sdlc-cli/src/cmd/init.rs
```

Expected: each grep returns ≥ 1.

### QA-4: summary.md format present

```bash
grep -c "summary.md" crates/sdlc-cli/src/cmd/init.rs
grep -c "PassWithTasks\|Pass With Tasks" crates/sdlc-cli/src/cmd/init.rs
```

Expected: each grep returns ≥ 1.

### QA-5: PLAYBOOK variant mentions Mode A/B

```bash
grep -A 60 'SDLC_MILESTONE_UAT_PLAYBOOK' crates/sdlc-cli/src/cmd/init.rs | grep -c "Mode A\|Mode B"
```

Expected: ≥ 1.

### QA-6: SKILL variant mentions Mode A/B

```bash
grep -A 30 'SDLC_MILESTONE_UAT_SKILL' crates/sdlc-cli/src/cmd/init.rs | grep -c "Mode A\|Mode B"
```

Expected: ≥ 1.

### QA-7: sdlc update installs correctly

```bash
sdlc update && grep -c "Mode A" ~/.claude/commands/sdlc-milestone-uat.md
```

Expected: exits 0, grep count ≥ 1.

## Pass Criteria

All 7 checks pass. No Rust compiler warnings or errors.
