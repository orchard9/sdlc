use crate::cmd::init::registry::CommandDef;

const SDLC_PRESSURE_TEST_COMMAND: &str = r#"---
description: Pressure-test a milestone against user perspectives — are we building what users actually want? Autonomously edits vision, features, acceptance tests, and creates tasks for gaps.
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-pressure-test

Pressure-test a milestone's direction against real user perspectives. This is not a code review or quality gate — it's a "are we solving the right problem?" check. Runs empathy interviews, identifies gaps between what's planned and what users need, and autonomously edits project docs to align them.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## When to Use

- Before starting implementation on a milestone (ideal)
- When a milestone feels off but you can't articulate why
- After a UAT failure that suggests we built the wrong thing
- When the team is building features no one asked for

## Ethos

- **Users over builders.** What we want to build matters less than what users need.
- **Edit, don't report.** This command produces changes, not a report that sits unread.
- **Conflicts are gold.** When user perspectives disagree with what's planned, that's the most valuable signal.
- **Always forward.** We add tasks, sharpen descriptions, and adjust acceptance criteria. The state machine moves forward.

---

## Steps

### 1. Load the milestone

```bash
sdlc milestone info <slug> --json
```

Extract title, vision, features, acceptance_test. If vision is empty, note as critical gap.

### 2. Load all features in the milestone

For each feature slug:
```bash
sdlc feature show <feature-slug>
```

Build a map of titles, descriptions, phases, existing specs, and tasks.

### 3. Identify user perspectives

Identify 3-5 specific user personas. Not abstract "users" — specific people in specific situations.

**Always include:**
1. The primary user (hands on keyboard daily)
2. Someone affected indirectly (downstream, ops, support)
3. A skeptic or reluctant adopter
4. A new/first-time user encountering this for the first time

### 4. Run empathy interviews (parallel)

For each perspective, conduct a deep interview:
- **Context**: typical day interacting with what this milestone delivers
- **Needs**: what problem it solves, what success looks like
- **Friction**: what would cause frustration or abandonment
- **Gaps**: what's missing from the planned features
- **Acceptance**: how they would test whether it delivers value

### 5. Synthesize findings

Analyze: alignments, conflicts, gaps, overbuilding, acceptance gaps.

### 6. Autonomous edits

#### A. Sharpen milestone vision
```bash
sdlc milestone update <slug> --vision "<sharpened vision>"
```

#### B. Update feature descriptions
```bash
sdlc feature update <feature-slug> --description "<user-aligned description>"
```

#### C. Add missing features
```bash
sdlc feature create <new-slug> --title "<title>" --description "<description>"
sdlc milestone add-feature <milestone-slug> <new-slug>
```

#### D. Create tasks for gaps
```bash
sdlc task add <feature-slug> --title "[user-gap] <specific gap to address>"
```

#### E. Update acceptance test
```bash
sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance_test.md
```

### 7. Commit and report

Print the pressure test report with perspectives consulted, edits made, conflicts surfaced, and overbuilding warnings.

---

### 8. Next

| Outcome | Next |
|---|---|
| Edits made, features in draft | `**Next:** /sdlc-run <first-feature-slug>` |
| New features created | `**Next:** /sdlc-run <new-feature-slug>` |
| Major direction change needed | `**Next:** /sdlc-plan` with revised plan |
| Milestone well-aligned | `**Next:** /sdlc-milestone-uat <slug>` |
"#;

const SDLC_PRESSURE_TEST_PLAYBOOK: &str = r#"# sdlc-pressure-test

Use this playbook to pressure-test a milestone against user perspectives.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load the milestone: `sdlc milestone info <slug> --json`.
2. Load all features: `sdlc feature show <feature-slug>` for each.
3. Identify 3-5 specific user personas (primary, indirect, skeptic, first-time).
4. Run empathy interviews for each perspective (context, needs, friction, gaps, acceptance).
5. Synthesize findings: alignments, conflicts, gaps, overbuilding, acceptance gaps.
6. Make autonomous edits:
   a. Sharpen vision: `sdlc milestone update <slug> --vision "<vision>"`.
   b. Update descriptions: `sdlc feature update <slug> --description "<desc>"`.
   c. Add missing features: `sdlc feature create` + `sdlc milestone add-feature`.
   d. Create gap tasks: `sdlc task add <slug> --title "[user-gap] <gap>"`.
   e. Update acceptance test: `sdlc milestone set-acceptance-test`.
7. Report: perspectives consulted, edits made, conflicts surfaced.

## Key Rules

- Users over builders. What we want to build matters less than what users need.
- Edit, don't report. Findings become changes to vision, features, tasks.
- Conflicts are gold. Don't smooth over disagreements — surface them.
"#;

const SDLC_PRESSURE_TEST_SKILL: &str = r#"---
name: sdlc-pressure-test
description: Pressure-test a milestone against user perspectives. Use when validating that a milestone builds what users actually want.
---

# SDLC Pressure Test Skill

Use this skill to pressure-test a milestone against user perspectives.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load milestone and its features from sdlc.
2. Identify 3-5 specific user personas (primary, indirect, skeptic, first-time).
3. Run empathy interviews for each perspective.
4. Synthesize: alignments, conflicts, gaps, overbuilding.
5. Make autonomous edits: sharpen vision, update descriptions, add features, create `[user-gap]` tasks, update acceptance test.
6. Report perspectives consulted, edits made, and conflicts surfaced.
"#;

pub static SDLC_PRESSURE_TEST: CommandDef = CommandDef {
    slug: "sdlc-pressure-test",
    claude_content: SDLC_PRESSURE_TEST_COMMAND,
    gemini_description: "Pressure-test a milestone against user perspectives",
    playbook: SDLC_PRESSURE_TEST_PLAYBOOK,
    opencode_description: "Pressure-test a milestone against user perspectives",
    opencode_hint: "<milestone-slug>",
    skill: SDLC_PRESSURE_TEST_SKILL,
};
