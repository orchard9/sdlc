use crate::cmd::init::registry::CommandDef;

const SDLC_PONDER_COMMIT_COMMAND: &str = r#"---
description: Commit to a pondered idea — crystallize it into milestones and features via sdlc-plan
argument-hint: <ponder-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-ponder-commit

Commit to a pondered idea. Reads the entire scrapbook, synthesizes with the recruited
thought partners using the feature-shaping protocol, and produces milestones, features,
and tasks that enter the state machine. The bridge from sketchbook to blueprint.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Prerequisites

A ponder entry should have enough substance to commit. Not a rigid checklist, but assess:

- Is the problem understood? (not just the solution)
- Have user perspectives been considered?
- Are the key technical decisions made?
- Is the scope defined?

If substance is thin, say so and suggest `/sdlc-ponder <slug>` to continue exploring.

---

## Steps

### 1. Load the scrapbook

```bash
sdlc ponder show <slug> --json
```

Read every artifact in the scrapbook. Read the team definitions. Build full context.

### 2. Load existing sdlc state

```bash
sdlc milestone list --json
sdlc feature list --json
```

Understand what already exists — avoid duplicating milestones or features.

### 3. Synthesize

With the full scrapbook and team context, determine the right structure:

**Small idea** (single capability, fits in one feature) →
- One feature, possibly added to an existing milestone
- Tasks decomposed from the exploration/decisions artifacts

**Medium idea** (multiple related capabilities) →
- One milestone with 2-5 features
- Vision synthesized from the problem framing and user perspectives

**Large idea** (significant initiative, multiple delivery phases) →
- Multiple milestones with clear ordering
- Each milestone has a user-observable goal

Present the proposed structure to the user.

### 4. Produce the plan

Write a structured plan to the scrapbook:

```bash
sdlc ponder capture <slug> --file /tmp/<slug>-plan.md --as plan.md
```

### 5. Distribute via sdlc-plan

Feed the plan into the state machine using the `/sdlc-plan` flow.

### 6. Update the ponder entry

```bash
sdlc ponder update <slug> --status committed
```

Record which milestones were created (update `committed_to` in manifest).

### 7. Report

Show what was created: milestones, features, tasks. Link back to the scrapbook.

---

### 8. Next

| Outcome | Next |
|---|---|
| Single feature created | `**Next:** /sdlc-run <feature-slug>` |
| Milestone created | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Multiple milestones | `**Next:** /sdlc-pressure-test <first-milestone-slug>` |
| Plan needs refinement | `**Next:** /sdlc-ponder <slug>` (back to exploring) |
"#;

const SDLC_PONDER_COMMIT_PLAYBOOK: &str = r#"# sdlc-ponder-commit

Crystallize a pondered idea into milestones and features.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load scrapbook: `sdlc ponder show <slug> --json`. Read all artifacts.
2. Load existing state: `sdlc milestone list --json`, `sdlc feature list --json`.
3. Assess readiness: problem understood? users considered? scope defined?
4. Synthesize: small → feature, medium → milestone + features, large → multiple milestones.
5. Write plan: `sdlc ponder capture <slug> --file /tmp/plan.md --as plan.md`.
6. Feed into state machine via `/sdlc-plan`.
7. Update: `sdlc ponder update <slug> --status committed`.
8. Report what was created. **Next:** pressure-test or run.
"#;

const SDLC_PONDER_COMMIT_SKILL: &str = r#"---
name: sdlc-ponder-commit
description: Crystallize a pondered idea into milestones and features. Use when an idea is ready to enter the state machine.
---

# SDLC Ponder Commit Skill

Use this skill to commit a pondered idea into the state machine.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load scrapbook: `sdlc ponder show <slug> --json`.
2. Load existing state: `sdlc milestone list --json`, `sdlc feature list --json`.
3. Assess readiness. If thin, suggest `/sdlc-ponder <slug>` instead.
4. Synthesize into milestones/features/tasks.
5. Write plan and feed via `/sdlc-plan`.
6. `sdlc ponder update <slug> --status committed`.
7. Report. **Next:** pressure-test or run.
"#;

pub static SDLC_PONDER_COMMIT: CommandDef = CommandDef {
    slug: "sdlc-ponder-commit",
    claude_content: SDLC_PONDER_COMMIT_COMMAND,
    gemini_description: "Crystallize a pondered idea into milestones and features",
    playbook: SDLC_PONDER_COMMIT_PLAYBOOK,
    opencode_description: "Crystallize a pondered idea into milestones and features",
    opencode_hint: "<ponder-slug>",
    skill: SDLC_PONDER_COMMIT_SKILL,
};
