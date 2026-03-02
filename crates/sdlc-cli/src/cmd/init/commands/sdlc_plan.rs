use crate::cmd::init::registry::CommandDef;

const SDLC_PLAN_COMMAND: &str = r#"---
description: Distribute a plan — week-by-week brief, task dump, or design doc — into sdlc milestones, features, and tasks. Idempotent: re-running refines what exists, never duplicates.
argument-hint: [--file <path>] or paste plan content inline
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-plan

Takes a body of work and distributes it into the sdlc structure. Creates milestones, features, and tasks where they don't exist. Refines them where they do. Running it again with a tweaked plan is safe and correct — the second run adjusts, not piles on.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Idempotency Contract

This is the most important property of this command. Every operation must be safe to repeat:

- **Milestones** — slug derived deterministically from the plan. If the slug already exists: update title, vision, and acceptance test. Never create a duplicate.
- **Features** — slug derived deterministically. If the slug already exists: update title and description. Never create a duplicate.
- **Milestone↔Feature links** — `sdlc milestone add-feature` is already idempotent. Run it unconditionally.
- **Tasks** — before adding, search for an existing task with a matching title in that feature. If found: skip. Never duplicate tasks.

Slug derivation must be deterministic: same plan text → same slugs every time. Lowercase, spaces → hyphens, strip punctuation, max 40 chars.

---

## Phase 1: Load Current State (parallel)

Run both simultaneously:

```bash
sdlc milestone list --json
sdlc feature list --json
```

Build a registry of existing milestones and features.

---

## Phase 2: Parse and Map

Read the plan. Produce a structured mapping before touching anything. Print it.

### What becomes a milestone
A milestone is a coherent unit of delivery with a user-observable goal, verifiable deliverables, and multiple related features.

### What becomes a feature
A feature is a semantically cohesive unit that ships together as one observable capability. Group related task-list items into one feature.

### What becomes a task
Individual implementation steps within a feature.

### Vision derivation
Synthesize the milestone goals into one sentence: what can a user do when this ships?

### Acceptance test derivation
Convert deliverables to a `- [ ]` checklist. Write it as `/tmp/<slug>_acceptance_test.md`.

---

## Phase 3: Execute (parallel agents per milestone)

Spawn one agent per milestone. Each agent:

### Step 1: Create or update the milestone
```bash
sdlc milestone create <slug> --title "<title>"
sdlc milestone update <slug> --vision "<derived vision>"
sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance_test.md
```

### Step 2: For each feature (sequential within agent)
```bash
sdlc feature create <slug> --title "<title>" --description "<description>"
sdlc milestone add-feature <milestone-slug> <feature-slug>
```

### Step 3: For each task in the feature
Check for duplicates with `sdlc task search`, then:
```bash
sdlc task add <feature-slug> "<title>"
```

---

## Phase 4: Summary

Print results: milestones created/updated, features created/updated, tasks added/skipped.

**Next:** `/sdlc-focus`

---

## Notes

- Features that exist but aren't in any milestone are re-linked to the correct milestone.
- If the plan has no explicit structure, derive milestone boundaries from semantic groupings.
- Lean toward fewer, larger milestones. A milestone should ship something a user can experience.
"#;

const SDLC_PLAN_PLAYBOOK: &str = r#"# sdlc-plan

Use this playbook to distribute a plan into sdlc milestones, features, and tasks.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Steps

1. Load current state: `sdlc milestone list --json` and `sdlc feature list --json`.
2. Parse the plan and produce a structured mapping (milestones → features → tasks).
3. For each milestone:
   a. Create or update: `sdlc milestone create <slug> --title "<title>"`.
   b. Set vision: `sdlc milestone update <slug> --vision "<vision>"`.
   c. Set acceptance test: `sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance_test.md`.
4. For each feature:
   a. Create or update: `sdlc feature create <slug> --title "<title>" --description "<desc>"`.
   b. Link: `sdlc milestone add-feature <milestone-slug> <feature-slug>`.
5. For each task: check for duplicates with `sdlc task search`, then `sdlc task add`.
6. Report: milestones created/updated, features created/updated, tasks added/skipped.

## Key Rules

- Idempotent: re-running refines, never duplicates.
- Slug derivation must be deterministic (lowercase, hyphens, max 40 chars).
- Group related items into cohesive features — don't make every line item a feature.
"#;

const SDLC_PLAN_SKILL: &str = r#"---
name: sdlc-plan
description: Distribute a plan into sdlc milestones, features, and tasks. Use when decomposing a roadmap or plan into trackable work.
---

# SDLC Plan Skill

Use this skill to distribute a plan into sdlc milestones, features, and tasks.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Workflow

1. Load current state: `sdlc milestone list --json` and `sdlc feature list --json`.
2. Parse the plan into milestones → features → tasks.
3. Create/update milestones with vision and acceptance tests.
4. Create/update features and link to milestones.
5. Add tasks, checking for duplicates first.
6. Report: counts of created, updated, and skipped items.

## Key Rule

Idempotent — re-running refines what exists, never duplicates.
"#;

pub static SDLC_PLAN: CommandDef = CommandDef {
    slug: "sdlc-plan",
    claude_content: SDLC_PLAN_COMMAND,
    gemini_description: "Distribute a plan into sdlc milestones, features, and tasks",
    playbook: SDLC_PLAN_PLAYBOOK,
    opencode_description: "Distribute a plan into sdlc milestones, features, and tasks",
    opencode_hint: "[--file <path>]",
    skill: SDLC_PLAN_SKILL,
};
