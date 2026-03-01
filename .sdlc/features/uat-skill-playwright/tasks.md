# Task Breakdown: Update sdlc-milestone-uat Skill with Playwright Support

## Tasks

### T1: Rewrite SDLC_MILESTONE_UAT_COMMAND with Mode A/B logic

Edit the `SDLC_MILESTONE_UAT_COMMAND` constant in `crates/sdlc-cli/src/cmd/init.rs`.

**Changes:**
- Update `allowed-tools` frontmatter to include Playwright MCP tools for Mode B browser navigation
- Add Mode A section: detect spec file, run `npx playwright test --reporter=json`, parse results, classify failures (code bug vs selector break), create tasks for bugs, fix and rerun for selector breaks
- Add Mode B section: read acceptance_test.md, use Playwright MCP browser tools to navigate each checklist item, write `frontend/e2e/milestones/<slug>.spec.ts`, run spec, fix selectors until passing, proceed to Mode A synthesis
- Add summary.md write step with specified format (Verdict, Tests, Tasks created, Results, Failures)
- Preserve existing uat_results.md and `sdlc milestone complete` lifecycle steps
- Keep ethos section intact

### T2: Rewrite SDLC_MILESTONE_UAT_PLAYBOOK (Gemini/OpenCode variant)

Edit `SDLC_MILESTONE_UAT_PLAYBOOK` constant in `crates/sdlc-cli/src/cmd/init.rs`.

**Changes:**
- Add Mode A / Mode B decision step at the top of the Steps section
- Include Playwright test execution command for Mode A
- Include spec generation flow for Mode B
- Include summary.md write step
- Keep concise format (~50 lines)

### T3: Rewrite SDLC_MILESTONE_UAT_SKILL (Agents SKILL.md variant)

Edit `SDLC_MILESTONE_UAT_SKILL` constant in `crates/sdlc-cli/src/cmd/init.rs`.

**Changes:**
- Add Mode A / Mode B decision in Workflow section
- Reference Playwright test execution
- Reference summary.md output
- Keep minimal format (~25 lines)

### T4: Verify build and install updated skill

- Run `SDLC_NO_NPM=1 cargo build --all` — must compile cleanly
- Run `sdlc update` to install updated skill to `~/.claude/commands/sdlc-milestone-uat.md`
- Verify Mode A / Mode B language is present in the installed file
