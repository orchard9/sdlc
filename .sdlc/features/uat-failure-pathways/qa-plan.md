# QA Plan: UAT Failure Triage and 3 Pathways

## Scope

All changes are Rust string constants (template text). QA focuses on:
1. Build and clippy correctness
2. Command installation verification via `sdlc update`
3. Template content correctness (triage logic, pathway steps, final report table)
4. Guidance table completeness

No runtime UAT tests are applicable — this feature modifies agent instruction text, not executable logic.

## QA Checklist

### 1. Build Verification

- [ ] `SDLC_NO_NPM=1 cargo build --all` exits 0
- [ ] `cargo clippy --all -- -D warnings` exits 0 with no warnings
- [ ] `SDLC_NO_NPM=1 cargo test --all` exits 0

### 2. Command Installation

- [ ] `sdlc update` completes without error
- [ ] `~/.claude/commands/sdlc-milestone-uat.md` exists and contains "Step 5 — Triage failures"
- [ ] `~/.claude/commands/sdlc-recap.md` exists and contains "Working On"
- [ ] `~/.gemini/commands/sdlc-recap.toml` exists
- [ ] `~/.agents/skills/sdlc-recap/SKILL.md` exists

### 3. Template Content: sdlc-milestone-uat

- [ ] Step 5 exists with triage classification table (Fixable / Escalation / Complex)
- [ ] Step 6 (Pathway 1) exists with fix + rerun + max-2-cycles logic
- [ ] Step 7 (Pathway 2) exists with `sdlc escalate create` and `uat/fail` call
- [ ] Step 8 (Pathway 3) exists with `sdlc ponder create`, commit, and `uat/fail` call
- [ ] Step 9 (Final report) has 5 verdicts: Pass, PassWithTasks, FixedAndPassed, Escalated, Recapped
- [ ] Pathway 3 ends with `**Next:** /sdlc-ponder <first-ponder-slug>`
- [ ] Pathway 2 ends with `**Next:** resolve escalation <id>, then /sdlc-milestone-uat <slug>`
- [ ] Old "Fix the feature tasks, then re-run this command" language is removed

### 4. Template Content: sdlc-recap

- [ ] Command lists 4 sections: Working On / Completed / Remaining / Forward Motion
- [ ] Forward Motion section describes creating tasks, escalations, or ponder entries
- [ ] Always ends with exactly one `**Next:**` line
- [ ] Playbook and Skill variants are consistent with the Claude Command

### 5. Guidance Table

- [ ] `GUIDANCE_MD_CONTENT` in `templates.rs` includes `/sdlc-recap` row
- [ ] After `sdlc update`, guidance table in the installed commands references `sdlc-recap`

### 6. Migration List

- [ ] `migrate_legacy_project_scaffolding` file lists include `sdlc-recap.md` and `sdlc-recap.toml` (or equivalent)

## Pass Criteria

All checklist items must be checked before marking QA results as approved.
