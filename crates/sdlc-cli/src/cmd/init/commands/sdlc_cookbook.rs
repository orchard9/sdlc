use crate::cmd::init::registry::CommandDef;

const SDLC_COOKBOOK_COMMAND: &str = r#"---
description: Create developer-scenario cookbook recipes that prove a milestone delivers real value — goals not features, promise not tickets
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-cookbook

Create developer-scenario cookbook recipes for a milestone. Cookbooks prove milestones deliver meaningful, usable capability — not just that features pass their tests. UAT asks "does the feature work?" while cookbooks ask "can a developer actually accomplish something?"

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Ethos

- **Goals not features.** Recipes are named after what a developer accomplishes, not what system components are exercised. "Bootstrap a project with AI agents ready" not "test skeleton installer."
- **Promise validation.** The recipe set proves the milestone's stated promise. Read the vision, not the ticket backlog.
- **Replayable by strangers.** Every recipe runs from a clean state with exact commands. No assumed state.
- **Edit don't report.** This command produces recipe files, not a report that sits unread.

---

## Steps

### 1. Load the milestone

```bash
sdlc milestone info <slug> --json
```

Extract title, vision, features, acceptance_test. The vision is your north star — extract the promise in one sentence.

### 2. Load all features in the milestone

For each feature slug:
```bash
sdlc feature show <feature-slug>
```

Understand what was built — specs, designs, implementation status. Features are means; the promise is the end.

### 3. Identify developer personas

Identify 1-3 developer personas who would exercise this milestone:
- **Primary builder** — hands on keyboard daily, building with this tool
- **First-timer** — encountering this for the first time, following docs
- **Integration dev** — wiring this into an existing system or pipeline

Different personas reveal different recipes. A first-timer reveals setup friction. An integration dev reveals API assumptions.

### 4. Draft recipe titles

Generate 3-5 recipe candidates. Each title must be:
- A developer goal in plain language (action verb + object)
- Achievable using only what the milestone delivers
- Independently runnable from a clean state

**Reject** recipes that could be replaced by a unit test, have "verify" or "test" as the primary verb, require state from a previous recipe, or prove only that the system doesn't crash.

**Accept** recipes that would appear in a "getting started" guide, represent real workflows, and would make a skeptic say "okay, this actually works."

### 5. Write recipe files

Write each recipe to `.cookbook/recipes/<milestone-slug>/recipe-NNN-<goal-slug>.md`:

```markdown
# Recipe: [Developer Goal in Plain Language]

## Goal
One sentence: what a developer is trying to accomplish.

## What It Proves
Why this matters. Connect explicitly back to the milestone's promise.

## Personas
Which developer persona(s) this recipe serves.

## Prerequisites
What state the world needs to be in. Keep minimal. Create all fixtures inline.

## Steps
Exact commands a developer types, in order. Do not describe commands — write the commands.

## Expected
- Key output lines (exact text or pattern)
- Files that MUST exist after the recipe completes
- Files that MUST NOT exist

## Verdict Criteria
How to evaluate: PASS, PARTIAL (what worked/didn't), FAIL (what broke).
```

### 6. Write cookbook infrastructure

- Write `.cookbook/README.md` if missing (what it is, how to run, where results live, how to add recipes)
- Ensure `.cookbook/runs/` is in `.gitignore` (results are ephemeral, never committed)

### 7. Acid test

Before finishing, challenge the full recipe set:

1. **Goal check** — Is each recipe named after what a developer accomplishes, or what the system does?
2. **Promise check** — Does this recipe set prove the milestone's stated promise? Every part of the promise needs at least one recipe.
3. **Replayability check** — Can someone with a clean machine run every recipe without asking questions?
4. **Sufficiency check** — Would a skeptic, after running these, agree the milestone succeeded?

Remove or revise any recipe that fails. Three strong recipes beat five weak ones.

---

### 8. Next

**Next:** `/sdlc-cookbook-run <milestone-slug>`
"#;

const SDLC_COOKBOOK_PLAYBOOK: &str = r#"# sdlc-cookbook

Create developer-scenario cookbook recipes that prove a milestone delivers real value.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load the milestone: `sdlc milestone info <slug> --json`. Extract the vision/promise.
2. Load all features: `sdlc feature show <feature-slug>` for each.
3. Identify 1-3 developer personas (primary builder, first-timer, integration dev).
4. Draft 3-5 recipe titles framed as developer goals (not feature names).
5. Write recipe files to `.cookbook/recipes/<milestone-slug>/recipe-NNN-<goal-slug>.md`.
   - Sections: Goal, What It Proves, Personas, Prerequisites, Steps, Expected, Verdict Criteria.
6. Write `.cookbook/README.md` if missing. Ensure `.cookbook/runs/` is in `.gitignore`.
7. Acid test: goal check, promise check, replayability check, sufficiency check.

## Key Rules

- Goals not features. Name recipes after what developers accomplish.
- Promise validation. Recipe set proves the milestone's stated promise.
- Replayable by strangers. Clean state, exact commands, inline fixtures.
- Three strong recipes beat five weak ones.
"#;

const SDLC_COOKBOOK_SKILL: &str = r#"---
name: sdlc-cookbook
description: Create developer-scenario cookbook recipes for a milestone. Use when proving a milestone delivers meaningful, usable capability — not just that features pass tests.
---

# SDLC Cookbook Skill

Use this skill to create developer-scenario cookbook recipes that prove a milestone delivers real value.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load milestone and extract the vision/promise: `sdlc milestone info <slug> --json`.
2. Load all features: `sdlc feature show <feature-slug>` for each.
3. Identify 1-3 developer personas (primary builder, first-timer, integration dev).
4. Draft 3-5 recipe titles as developer goals (not feature names).
5. Write recipes to `.cookbook/recipes/<milestone-slug>/recipe-NNN-<goal-slug>.md`.
6. Write `.cookbook/README.md` if missing. Add `.cookbook/runs/` to `.gitignore`.
7. Acid test: goal check, promise check, replayability check, sufficiency check.

## Key Rules

- Recipes named after developer goals, not system components.
- Recipe set proves the milestone's stated promise.
- Every recipe runnable from clean state with exact commands.
- Three strong recipes beat five weak ones.
"#;

pub static SDLC_COOKBOOK: CommandDef = CommandDef {
    slug: "sdlc-cookbook",
    claude_content: SDLC_COOKBOOK_COMMAND,
    gemini_description: "Create developer-scenario cookbook recipes for a milestone",
    playbook: SDLC_COOKBOOK_PLAYBOOK,
    opencode_description: "Create developer-scenario cookbook recipes for a milestone",
    opencode_hint: "<milestone-slug>",
    skill: SDLC_COOKBOOK_SKILL,
};
