use crate::cmd::init::registry::CommandDef;

const SDLC_RUN_WAVE_COMMAND: &str = r#"---
description: Execute Wave 1 features in parallel, then advance to the next wave
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent
---

# sdlc-run-wave

Execute the current wave of a milestone in parallel, then re-run prepare to advance to the next wave.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Resolve the milestone slug

Use `$ARGUMENTS` as the slug. If none provided, run:
```bash
sdlc project prepare --json
```
and read `milestone_slug` from the output.

### 2. Check for wave plan

Read `.sdlc/milestones/<slug>/wave_plan.yaml`. If missing, stop and tell the user:

> Wave plan not found. Run `/sdlc-prepare <slug>` first to generate it.

### 3. Get the authoritative current wave

Re-run prepare to get live state — this is always authoritative:
```bash
sdlc project prepare --milestone <slug> --json
```

Wave 1 of the prepare output is the current wave (features not yet done). The wave_plan.yaml is the structural record; prepare output is the live state.

### 4. Summarize the wave

Print:
- Wave number and label
- Feature count
- For each feature: slug, phase, next action
- Whether any features need worktrees (`needs_worktrees` flag)

### 5. Handle worktree features

If any Wave 1 features have `needs_worktrees: true`, print a notice for each:

> **Manual step required:** Feature `<slug>` needs a dedicated worktree.
> Run in a separate terminal: `/sdlc-run <slug>`

Skip these features from the parallel batch.

### 6. Execute remaining Wave 1 features in parallel

For each remaining feature in Wave 1, spawn a parallel Agent call running `/sdlc-run <feature-slug>`.

Use the Agent tool with multiple concurrent calls — one per feature. Do not run them sequentially.

Wait for all agents to complete.

### 7. Advance to next wave

After all Wave 1 agents complete, re-run:
```bash
sdlc project prepare --milestone <slug> --json
```

Check the result:
- **Waves remain** — loop back to step 3 and execute the next wave.
- **No waves remain, milestone is `Verifying`** — proceed to step 8.
- **Blockers surfaced** — stop and report them.

### 8. Run UAT automatically when all waves are done

When prepare returns no remaining waves and the milestone is `Verifying`, use the `Skill` tool to execute `sdlc-milestone-uat` with args `<slug>` — do **not** print a Next suggestion or stop here. The UAT skill will write results, execute `ponder milestone complete <slug>` on pass, and end with its own `**Next:**` line.

### 9. Next

Only print a `**Next:**` line if execution stopped before UAT:

| Outcome | Next |
|---|---|
| Blockers surfaced | `**Next:** Resolve blockers listed above, then /sdlc-run-wave <slug>` |
"#;

const SDLC_RUN_WAVE_PLAYBOOK: &str = r#"# sdlc-run-wave

Execute the current wave of a milestone in parallel, then advance to the next wave.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve slug from arguments; if missing, run `sdlc project prepare --json` and read `milestone_slug`.
2. Read `.sdlc/milestones/<slug>/wave_plan.yaml` — if missing, tell user to run `/sdlc-prepare <slug>` first.
3. Run `sdlc project prepare --milestone <slug> --json` — Wave 1 of this output is the authoritative current wave.
4. For features with `needs_worktrees: true`: print manual step instructions; skip from parallel batch.
5. Execute remaining Wave 1 features in parallel (spawn concurrent `/sdlc-run <slug>` calls).
6. After all complete, re-run `sdlc project prepare --milestone <slug> --json`.
7. If waves remain, loop back to step 3. If no waves remain (milestone `Verifying`), **execute the `sdlc-milestone-uat <slug>` slash command now** — do not print a Next suggestion or pause. If blockers surfaced, end: `**Next:** Resolve blockers, then /sdlc-run-wave <slug>`.
"#;

const SDLC_RUN_WAVE_SKILL: &str = r#"---
name: sdlc-run-wave
description: Execute Wave 1 features of a milestone in parallel, then advance to the next wave. Use after /sdlc-prepare.
---

# SDLC Run-Wave Skill

Execute the current milestone wave in parallel and advance.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Resolve slug; if missing run `sdlc project prepare --json` to find active milestone.
2. Check `.sdlc/milestones/<slug>/wave_plan.yaml` exists — if not, tell user to run `/sdlc-prepare <slug>`.
3. Run `sdlc project prepare --milestone <slug> --json`. Wave 1 is the current wave.
4. Skip features needing worktrees (print manual instructions). Execute the rest in parallel via `/sdlc-run <slug>`.
5. After all complete, re-run prepare. If waves remain, loop to step 3. If no waves (milestone `Verifying`), **execute `sdlc-milestone-uat <slug>` now** — do not pause or print a Next suggestion. If blockers: `**Next:** Resolve blockers, then /sdlc-run-wave <slug>`.
"#;

pub static SDLC_RUN_WAVE: CommandDef = CommandDef {
    slug: "sdlc-run-wave",
    claude_content: SDLC_RUN_WAVE_COMMAND,
    gemini_description: "Execute Wave 1 features in parallel, then advance to the next wave",
    playbook: SDLC_RUN_WAVE_PLAYBOOK,
    opencode_description: "Execute Wave 1 features in parallel, then advance to the next wave",
    opencode_hint: "<milestone-slug>",
    skill: SDLC_RUN_WAVE_SKILL,
};
