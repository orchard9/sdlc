use crate::cmd::init::registry::CommandDef;

const SDLC_COOKBOOK_RUN_COMMAND: &str = r#"---
description: Execute cookbook recipes for a milestone — run each scenario, record verdicts, create tasks for failures
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-cookbook-run

Execute cookbook recipes for a milestone and record the results. Be the developer. Run every step. Record honest verdicts. Failures become `[cookbook-gap]` tasks on the owning feature.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Ethos

- **Be the developer.** Run the exact commands in the recipe. Don't skip steps, don't improvise.
- **Never pause.** Execute all recipes in sequence. Stop only when all are done.
- **Always forward.** Failures create tasks. The state machine moves forward.
- **Honest verdicts.** A PARTIAL that documents what broke is worth more than a PASS that hides issues.

---

## Steps

### 1. Load recipes

```bash
ls .cookbook/recipes/<milestone-slug>/
```

Read all recipe files from `.cookbook/recipes/<milestone-slug>/`. If no recipes exist, stop and say: "No recipes found. Run `/sdlc-cookbook <milestone-slug>` first."

### 2. Load milestone context

```bash
sdlc milestone info <slug> --json
```

Map recipes to features — understand which feature each recipe exercises.

### 3. Create run directory

Create a timestamped run directory:
```
.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/
```

### 4. Execute each recipe

For each recipe file, in order:

1. **Read the recipe** — understand goal, prerequisites, steps, expected outcomes
2. **Run prerequisites** — execute setup commands, create fixtures
3. **Execute steps** — run each command exactly as written, capture output
4. **Evaluate expected** — check output against expected lines, verify files exist/don't exist
5. **Record verdict** — PASS, PARTIAL (what worked + what didn't), or FAIL (what broke)
6. **Write result file** — save as `<recipe-name>.result.md` in the run directory

### 5. Handle failures

On PARTIAL or FAIL:
```bash
sdlc task add <feature-slug> --title "[cookbook-gap] <recipe-name>: <failure summary>"
```

Create one task per failure, on the feature the recipe exercises.

### 6. Write summary

Write `summary.md` in the run directory:

```markdown
# Cookbook Run: <milestone-slug>

**Date:** <timestamp>
**Commit:** <git rev-parse HEAD>
**Environment:** <OS, arch>

## Results

| Recipe | Verdict | Notes |
|--------|---------|-------|
| recipe-001-... | PASS/PARTIAL/FAIL | ... |

## Overall: PASS / PARTIAL / FAIL

**What this confirms:**

**What is still open:**
```

### 7. Report

Print the summary with overall verdict, individual results, and tasks created for failures.

---

### 8. Next

| Outcome | Next |
|---|---|
| All PASS | `**Next:** /sdlc-milestone-verify <milestone-slug>` |
| Any FAIL/PARTIAL | `**Next:** /sdlc-run <failing-feature-slug>` |
"#;

const SDLC_COOKBOOK_RUN_PLAYBOOK: &str = r#"# sdlc-cookbook-run

Execute cookbook recipes for a milestone and record results.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load recipes from `.cookbook/recipes/<milestone-slug>/`.
2. Load milestone context: `sdlc milestone info <slug> --json`. Map recipes to features.
3. Create run dir: `.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/`.
4. For each recipe: run prerequisites, execute steps, evaluate expected, record verdict.
5. On PARTIAL/FAIL: `sdlc task add <feature-slug> --title "[cookbook-gap] <recipe>: <failure>"`.
6. Write `<recipe>.result.md` + `summary.md` in run directory.
7. Report overall verdict.

## Key Rules

- Be the developer. Run exact commands from the recipe.
- Honest verdicts. PARTIAL that documents what broke beats a PASS that hides issues.
- Failures create `[cookbook-gap]` tasks on the owning feature.
- All PASS → `/sdlc-milestone-verify <slug>`. Any FAIL → `/sdlc-run <failing-feature>`.
"#;

const SDLC_COOKBOOK_RUN_SKILL: &str = r#"---
name: sdlc-cookbook-run
description: Execute cookbook recipes for a milestone and record results. Use when validating that cookbook scenarios pass and creating tasks for failures.
---

# SDLC Cookbook Run Skill

Use this skill to execute cookbook recipes and record results for a milestone.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load recipes from `.cookbook/recipes/<milestone-slug>/`.
2. Load milestone context: `sdlc milestone info <slug> --json`. Map recipes to features.
3. Create run dir: `.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/`.
4. Execute each recipe: prerequisites, steps, evaluate expected, record verdict.
5. On failure: `sdlc task add <feature-slug> --title "[cookbook-gap] <recipe>: <failure>"`.
6. Write result files + summary in run directory.
7. Report overall verdict. All PASS → milestone-verify. Any FAIL → sdlc-run on failing feature.
"#;

pub static SDLC_COOKBOOK_RUN: CommandDef = CommandDef {
    slug: "sdlc-cookbook-run",
    claude_content: SDLC_COOKBOOK_RUN_COMMAND,
    gemini_description: "Execute cookbook recipes and record results for a milestone",
    playbook: SDLC_COOKBOOK_RUN_PLAYBOOK,
    opencode_description: "Execute cookbook recipes and record results for a milestone",
    opencode_hint: "<milestone-slug>",
    skill: SDLC_COOKBOOK_RUN_SKILL,
};
