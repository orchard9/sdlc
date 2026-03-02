use crate::cmd::init::registry::CommandDef;

const SDLC_HYPOTHETICAL_PLANNING_COMMAND: &str = r#"---
description: Build a hypothetical implementation plan in .sdlc/ — architecture, file manifest, perspective review, and READY/BLOCKED verdict
argument-hint: <feature-or-milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent
---

# sdlc-hypothetical-planning

Build a complete hypothetical plan for implementing a feature or milestone before writing a single line of production code. Produces a structured set of artifacts in `.sdlc/hypotheticals/<slug>/` and ends with a binary confidence verdict: READY (pass to `/sdlc-hypothetical-do`) or BLOCKED (list of unknowns to resolve).

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Resolve the slug

Use `$ARGUMENTS`. If it refers to a feature: `sdlc feature show <slug> --json`. If a milestone: `sdlc milestone info <slug> --json`.

Read all existing artifacts: `.sdlc/features/<slug>/spec.md`, `design.md`, `tasks.md`.

Create `.sdlc/hypotheticals/<slug>/` and write `manifest.yaml`:
```yaml
slug: <slug>
subject: <one-line description from spec or title>
status: in_progress
created_at: <ISO timestamp>
```

### 2. Architecture layer

Write `.sdlc/hypotheticals/<slug>/architecture.md`.

Cover:
- What changes at the system level (new subsystems, removed abstractions, boundary shifts)
- Central components (3–7 load-bearing pieces): what each owns, why it must change
- Interaction model after the change (data flows, API contracts, event sequences)
- What explicitly does NOT change (grounds scope)

Do not name individual files yet — reason about shapes and ownership.

### 3. Component breakdown

Write `.sdlc/hypotheticals/<slug>/components.md`.

For each central component: current state, required change, rationale for scope, downstream effects. Work inside-out: change the core, trace outward.

### 4. File manifest

Write `.sdlc/hypotheticals/<slug>/file-manifest.md`.

Sections: **Added** | **Modified** | **Removed** | **Unchanged (notable)**.

For each file: path, what changes, why. Include test files, migration files, config files — not just source. Every component from step 3 must map to at least one file.

### 5. Perspective review

Write `.sdlc/hypotheticals/<slug>/perspective-review.md`.

Review through four lenses:
1. **Correctness** — does architecture handle edge cases and failure modes? Missing or orphaned files?
2. **Coherence** — consistent scope, naming, no full rewrites disguised as edits?
3. **Completeness** — tests, migrations, config, docs accounted for?
4. **Risk** — highest-risk 2–3 files, hardest problem, silent-failure scenarios?

Resolve all issues found by updating earlier artifacts before proceeding.

### 6. Step back — confidence check

Before writing the verdict, challenge:
- Is this the right scope? Should it be smaller?
- Are there files I know exist but haven't read?
- What assumption in the architecture is least verified?
- What fact discovered during implementation would invalidate the plan?

### 7. Confidence verdict

Write `.sdlc/hypotheticals/<slug>/confidence.md`.

Verdict is **READY** or **BLOCKED** — binary, no middle ground.

**If READY:** confirm architecture is consistent, manifest covers all components, perspective review is clean, no unknowns.

**If BLOCKED:** produce a blocker table:
| # | Question | Why It Blocks | How to Resolve |

Always include: highest-risk file, hardest problem, silent-failure risk.

Update `manifest.yaml` status to `ready` or `blocked`.

### 8. Report

Print a summary: subject, verdict, file count (added/modified/removed), key risks.

**Next:**
- READY: `**Next:** /sdlc-hypothetical-do <slug>`
- BLOCKED: `**Next:** resolve blockers above, then /sdlc-hypothetical-planning <slug>`
"#;

const SDLC_HYPOTHETICAL_PLANNING_PLAYBOOK: &str = r#"# sdlc-hypothetical-planning

Build a hypothetical implementation plan in `.sdlc/hypotheticals/<slug>/` before writing production code.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve slug from arguments. Read `sdlc feature show <slug> --json` or `sdlc milestone info <slug> --json`. Read existing artifacts (spec.md, design.md, tasks.md).
2. Create `.sdlc/hypotheticals/<slug>/`. Write `manifest.yaml` with `status: in_progress`.
3. Write `architecture.md` — system-level changes, central components (3–7), interaction model, what doesn't change. No individual files yet.
4. Write `components.md` — for each component: current state, required change, scope rationale, downstream effects.
5. Write `file-manifest.md` — every file: Added / Modified / Removed / Unchanged. Include tests, migrations, configs. Every component must map to at least one file.
6. Write `perspective-review.md` — four lenses: Correctness, Coherence, Completeness, Risk. Resolve all issues found before continuing.
7. Step back: Is scope right? Any unread files? What assumption is least verified?
8. Write `confidence.md` — verdict is READY or BLOCKED (binary). BLOCKED requires a blocker table with question / why-it-blocks / how-to-resolve.
9. Update `manifest.yaml` status to `ready` or `blocked`.
10. End: READY → `**Next:** /sdlc-hypothetical-do <slug>` | BLOCKED → `**Next:** resolve blockers, then /sdlc-hypothetical-planning <slug>`
"#;

const SDLC_HYPOTHETICAL_PLANNING_SKILL: &str = r#"---
name: sdlc-hypothetical-planning
description: Build a hypothetical implementation plan in .sdlc/ — architecture, file manifest, perspective review, READY/BLOCKED verdict. Use before committing to implementation.
---

# SDLC Hypothetical Planning Skill

Pre-implementation confidence check: build the full picture before writing production code.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Read feature/milestone: `sdlc feature show <slug> --json` or `sdlc milestone info <slug> --json`. Read spec/design/tasks artifacts.
2. Create `.sdlc/hypotheticals/<slug>/`, write `manifest.yaml` (`status: in_progress`).
3. Write `architecture.md` — system changes, 3–7 central components, interaction model, explicit scope boundary.
4. Write `components.md` — each component: current state, change, rationale, effects.
5. Write `file-manifest.md` — every file added/modified/removed/unchanged. Include tests, migrations, configs.
6. Write `perspective-review.md` — Correctness, Coherence, Completeness, Risk. Resolve issues before proceeding.
7. Write `confidence.md` — binary READY or BLOCKED. BLOCKED requires blocker table.
8. Update `manifest.yaml` status. End: READY → `/sdlc-hypothetical-do <slug>` | BLOCKED → resolve then re-run.
"#;

pub static SDLC_HYPOTHETICAL_PLANNING: CommandDef = CommandDef {
    slug: "sdlc-hypothetical-planning",
    claude_content: SDLC_HYPOTHETICAL_PLANNING_COMMAND,
    gemini_description: "Build a hypothetical implementation plan with architecture, file manifest, and READY/BLOCKED verdict",
    playbook: SDLC_HYPOTHETICAL_PLANNING_PLAYBOOK,
    opencode_description: "Build a hypothetical implementation plan with architecture, file manifest, and READY/BLOCKED verdict",
    opencode_hint: "<feature-or-milestone-slug>",
    skill: SDLC_HYPOTHETICAL_PLANNING_SKILL,
};
