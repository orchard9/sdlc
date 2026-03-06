# Scope Assessment: What Actually Changes

## Template/Skill changes (the real work)

### 1. New: `sdlc-recap` command template
- Claude Code command: `~/.claude/commands/sdlc-recap.md`
- Gemini playbook: `~/.gemini/commands/sdlc-recap.toml`
- OpenCode: `~/.opencode/command/sdlc-recap.md`
- Agent Skills: `~/.agents/skills/sdlc-recap/SKILL.md`
- **Location in code**: New `const` in `crates/sdlc-cli/src/cmd/init/commands/` (new file `sdlc_recap.rs`)
- Register in all four `write_user_*` functions in `init.rs`

### 2. Modified: `sdlc-milestone-uat` command template
- Add Steps 5-8 (triage → pathway 1/2/3) to the Claude command
- Update playbook and skill variants
- **Location**: `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

### 3. Modified: GUIDANCE_MD_CONTENT
- Add `sdlc-recap` to the command reference table in `init.rs`

## Rust code changes: NONE

⚑  Decided: No new Rust code, no new endpoints, no new data types.

Why:
- Escalation system already exists (`sdlc escalate create`)
- Ponder create already exists (`sdlc ponder create`)
- Task add already exists (`sdlc task add`)  
- Status query already exists (`sdlc status --json`)
- The decision logic (which pathway?) belongs in skill text
- The recap synthesis is agent work, not library work

## Migration

- `migrate_legacy_project_scaffolding()` needs the new `sdlc-recap` filename
- `sdlc update` will install the new command on next run

## Features for milestone

1. **uat-failure-pathways**: Modify the UAT template to add triage + 3 pathways
2. **sdlc-recap-command**: Create the new sdlc-recap command template (4 platform variants)
3. Maybe: **recap-from-server**: Server route to run recap via UI button (optional, could be wave 2)

?  Open: Do we need a "Recap" button in the milestone UAT failure UI? Currently the frontend shows a failure badge via SSE. Could add a "Run Recap" button that triggers an agent run. But this adds a server endpoint — which contradicts the "no Rust changes" assessment. Park this for now.
