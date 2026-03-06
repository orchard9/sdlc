# Tasks: UAT Failure Triage and 3 Pathways

## T1: Update sdlc_milestone_uat.rs — triage classification + 3 pathways in COMMAND variant

Replace Steps 5–6 in `SDLC_MILESTONE_UAT_COMMAND` with Steps 5–9:
- Step 5: Triage each failure as Fixable / Escalation / Complex
- Step 6: Pathway 1 — Fix and Retry (max 2 cycles)
- Step 7: Pathway 2 — Escalate (tasks + escalation + uat/fail)
- Step 8: Pathway 3 — Recap and Propose (ponder entries + commit + uat/fail)
- Step 9: Final report table with 5 verdicts

File: `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

## T2: Update sdlc_milestone_uat.rs — playbook and skill variants

Update `SDLC_MILESTONE_UAT_PLAYBOOK` and `SDLC_MILESTONE_UAT_SKILL` to reflect the new triage + 3 pathways approach.

File: `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

## T3: Create sdlc_recap.rs — new /sdlc-recap command

Create `crates/sdlc-cli/src/cmd/init/commands/sdlc_recap.rs` with:
- `SDLC_RECAP_COMMAND` — Claude Code full command
- `SDLC_RECAP_PLAYBOOK` — Gemini/OpenCode concise variant
- `SDLC_RECAP_SKILL` — Agent Skills minimal SKILL.md
- `pub static SDLC_RECAP: CommandDef` struct

Command content must include:
1. Read `sdlc status --json`, milestone info, `git log --oneline -20`
2. Produce 4 sections: Working On / Completed / Remaining / Forward Motion
3. Forward Motion creates real artifacts: tasks, escalations, ponder entries
4. Always ends with exactly one `**Next:**` line

## T4: Register sdlc_recap in mod.rs and ALL_COMMANDS

- Add `mod sdlc_recap;` to `crates/sdlc-cli/src/cmd/init/commands/mod.rs`
- Add `&sdlc_recap::SDLC_RECAP` to `ALL_COMMANDS` in `mod.rs`

## T5: Add /sdlc-recap to GUIDANCE_MD_CONTENT table in templates.rs

Add row to the command reference table in `GUIDANCE_MD_CONTENT` (§6 "Using sdlc") in `crates/sdlc-cli/src/cmd/init/templates.rs`:
```
| /sdlc-recap | Synthesize project state and create forward-motion artifacts (Working On / Completed / Remaining / Forward Motion) |
```

## T6: Build verification and sdlc update

After all code changes:
1. `SDLC_NO_NPM=1 cargo build --all` — must succeed
2. `cargo clippy --all -- -D warnings` — must pass
3. `SDLC_NO_NPM=1 cargo test --all` — must pass
4. `sdlc update` — installs updated commands; verify `sdlc-recap.md` appears in `~/.claude/commands/`
