use crate::cmd::init::registry::CommandDef;

const SDLC_PREPARE_COMMAND: &str = r#"---
description: Pre-flight a milestone — align features with vision, fix gaps, write wave plan, mark ready to execute
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent
---

# sdlc-prepare

Pre-flight a milestone end-to-end: read the vision, audit every feature for alignment, fix structural gaps, write a wave plan, and mark the milestone prepared. This command makes real changes — it is not read-only.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Resolve the milestone slug

Use `$ARGUMENTS` as the slug. If none provided, run:
```bash
sdlc project prepare --json
```
and pick the active milestone from `milestone_slug` in the output.

### 2. Read the product vision

In order:
1. `docs/vision.md` — if it exists, read it
2. `CLAUDE.md` — read the `## Project` section
3. `README.md` — first two sections

Synthesize a one-paragraph vision statement to use as the alignment anchor.

### 3. Read milestone state

```bash
sdlc milestone info <slug> --json
```

Note: `vision`, `features` list, `prepared_at`.

### 4. Audit each feature for alignment

For each feature slug in the milestone:

```bash
sdlc feature show <slug> --json
```

Then read any existing artifacts:
- `.sdlc/features/<slug>/spec.md`
- `.sdlc/features/<slug>/design.md`
- `.sdlc/features/<slug>/tasks.md`

Check:
- Does the description exist and clearly connect to the milestone vision?
- Are tasks concrete and actionable (not vague placeholders)?
- Do dependency references point to real feature slugs?

### 5. Fix structural gaps

For each feature needing repair:

**Missing or weak description:**
```bash
sdlc feature update <slug> --description "<clear one-liner tied to the vision>"
```

**Broken dependency reference** (dep slug doesn't exist):
```bash
sdlc feature update <slug> --depends-on <correct-slug>
```

**Vague tasks** — rewrite with specific action verbs:
```bash
sdlc task update <slug> <task-id> --title "<specific action>"
```

**Features that don't belong** (contradict vision, wrong milestone) — archive them:
```bash
sdlc feature archive <slug>
```

### 6. Run prepare and build wave plan

```bash
sdlc project prepare --milestone <slug> --json
```

Parse the `waves` array. Write a wave plan file:

```bash
# Build wave_plan.yaml content from prepare output and write to the milestone dir
```

Wave plan format at `.sdlc/milestones/<slug>/wave_plan.yaml`:
```yaml
milestone: <slug>
waves:
  - number: 1
    label: Planning
    slugs: [feat-a, feat-b]
  - number: 2
    label: Implementation
    slugs: [feat-c]
```

Use wave labels from the prepare output if present; otherwise label Wave 1 `Planning`, Wave 2 `Implementation`, remaining waves `Wave N`.

### 7. Mark milestone prepared

```bash
sdlc milestone mark-prepared <slug>
```

### 8. Report

Print a summary:
1. **Vision** — the one-paragraph anchor used
2. **Fixes applied** — what was changed and why
3. **Wave Plan** — wave number, label, feature slugs, feature count
4. **Blocked features** — any features that couldn't be fixed (explain why)

### 9. Next

Always end with exactly one `**Next:**` line:

| Outcome | Next |
|---|---|
| Wave plan written, milestone prepared | `**Next:** /sdlc-run-wave <slug>` |
| Blockers remain after fixes | `**Next:** Resolve the blockers above, then re-run /sdlc-prepare <slug>` |
| All features already done (verifying) | `**Next:** /sdlc-milestone-uat <slug>` |
| Project idle (no active milestone) | `**Next:** /sdlc-ponder to start exploring ideas` |
"#;

const SDLC_PREPARE_PLAYBOOK: &str = r#"# sdlc-prepare

Pre-flight a milestone: align features with vision, fix gaps, write wave plan, mark prepared.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve slug from arguments; if missing, run `sdlc project prepare --json` and read `milestone_slug`.
2. Read product vision from `docs/vision.md`, `CLAUDE.md` §Project, and `README.md`.
3. Run `sdlc milestone info <slug> --json`. For each feature: `sdlc feature show <slug> --json` and read spec/design/tasks.md if present.
4. Fix structural gaps: missing descriptions (`sdlc feature update`), broken deps, vague tasks (`sdlc task update`), out-of-scope features (`sdlc feature archive`).
5. Run `sdlc project prepare --milestone <slug> --json`. Write `.sdlc/milestones/<slug>/wave_plan.yaml` from the `waves` array.
6. Run `sdlc milestone mark-prepared <slug>`.
7. Report: vision anchor, fixes applied, wave plan summary, any remaining blockers.
8. End: `**Next:** /sdlc-run-wave <slug>` (or `fix blockers then /sdlc-prepare` if blockers remain).
"#;

const SDLC_PREPARE_SKILL: &str = r#"---
name: sdlc-prepare
description: Pre-flight a milestone — align features with vision, fix gaps, write wave plan, mark prepared. Use before executing a milestone.
---

# SDLC Prepare Skill

Pre-flight a milestone end-to-end and mark it ready for parallel execution.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Resolve slug; if missing run `sdlc project prepare --json` to find active milestone.
2. Read vision from `docs/vision.md`, `CLAUDE.md`, `README.md`.
3. Audit each feature: `sdlc feature show <slug> --json` + read artifacts. Fix descriptions, tasks, deps.
4. Run `sdlc project prepare --milestone <slug> --json`. Write `.sdlc/milestones/<slug>/wave_plan.yaml`.
5. Run `sdlc milestone mark-prepared <slug>`.
6. End: `**Next:** /sdlc-run-wave <slug>`.
"#;

pub static SDLC_PREPARE: CommandDef = CommandDef {
    slug: "sdlc-prepare",
    claude_content: SDLC_PREPARE_COMMAND,
    gemini_description:
        "Pre-flight a milestone — align features, fix gaps, write wave plan, mark prepared",
    playbook: SDLC_PREPARE_PLAYBOOK,
    opencode_description:
        "Pre-flight a milestone — align features, fix gaps, write wave plan, mark prepared",
    opencode_hint: "<milestone-slug>",
    skill: SDLC_PREPARE_SKILL,
};
