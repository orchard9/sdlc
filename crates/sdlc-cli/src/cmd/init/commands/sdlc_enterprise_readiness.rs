use crate::cmd::init::registry::CommandDef;

const SDLC_ENTERPRISE_READINESS_COMMAND: &str = r#"---
description: Analyze a project for enterprise-grade production readiness and distribute findings into sdlc milestones, features, and tasks via sdlc-plan — or add to existing milestones and update active tasks
argument-hint: [--stage <mvp|production|scale|enterprise>] [--into <milestone-slug>]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-enterprise-readiness

Run an enterprise readiness analysis against the current project and translate findings into sdlc structure. The output is not a report — it's milestones, features, and tasks that enter the state machine and get built.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

Three modes of operation:

1. **Create new milestones** (default) — groups findings into `ops-*` milestones and feeds them through sdlc-plan
2. **Add to existing milestone** (`--into <slug>`) — adds findings as features/tasks to an existing milestone
3. **Update active tasks** — when findings overlap with in-progress features, adds `[ops-gap]` tasks

## Ethos

- **Gaps become milestones, not reports.** Every finding either becomes a feature or gets explicitly deferred with rationale.
- **Build for the next stage, not three ahead.** MVP projects don't need multi-region. Scope to what matters now.
- **Blast radius drives priority.** A missing timeout can crash the service (P0). A missing Grafana panel is annoying (P3).

---

## Steps

### 1. Determine current and target stage

Parse `$ARGUMENTS` for `--stage`. If not provided, infer from project signals.

| Stage | Signals | Next Stage |
|---|---|---|
| **MVP Pilot** | No CI/CD, no monitoring, manual deploys | Production |
| **Production** | CI/CD exists, basic health checks, some docs | Scale |
| **Scale** | Monitoring, automated ops, multi-customer | Enterprise |
| **Enterprise** | Compliance artifacts, DR runbooks, SLAs | Maintenance |

### 2. Load existing sdlc state

```bash
sdlc milestone list --json
sdlc feature list --json
```

### 3. Run enterprise readiness analysis

Launch expert agents in parallel:
- **A. Storage/Data** — backup, recovery, data integrity, persistence
- **B. Operations** — deployment, monitoring, alerting, runbooks
- **C. Security** — TLS, auth, rate limiting, secrets management

### 4. Route findings based on mode

#### Mode A: Add to existing milestone (`--into <slug>`)
Add `[ops-gap]` tasks to existing features or create new features within the milestone.

#### Mode B: Update active tasks (automatic)
Scan existing features for overlap and add `[ops-gap]` tasks where applicable.

#### Mode C: Create new milestones (default)
Group remaining gaps into `ops-*` milestones (ops-ship-blockers, ops-production-hardening, etc.).

### 5. Synthesize remaining gaps into a plan

Assign priority (P0-P3), group into milestones, decompose into features and tasks.

### 7. Produce the plan document (Mode C only)

Write to `/tmp/enterprise-readiness-plan.md` and feed through `/sdlc-plan`.

### 9. Report

Print enterprise readiness report with distributed gaps, absorbed tasks, deferred items, and expert consensus.

---

### 10. Next

| Outcome | Next |
|---|---|
| Ship blockers created (Mode C) | `**Next:** /sdlc-run <first-ops-feature-slug>` |
| Added to milestone (Mode A) | `**Next:** /sdlc-run <first-new-feature-slug>` |
| Tasks added to active features (Mode B) | `**Next:** /sdlc-status` |
| Already enterprise-ready | `**Next:** /sdlc-status` |
"#;

const SDLC_ENTERPRISE_READINESS_PLAYBOOK: &str = r#"# sdlc-enterprise-readiness

Use this playbook to analyze a project for enterprise readiness and distribute findings into sdlc.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Determine current and target stage (MVP → Production → Scale → Enterprise).
2. Load existing state: `sdlc milestone list --json` and `sdlc feature list --json`.
3. Run analysis across three domains: Storage/Data, Operations, Security.
4. Route findings by mode:
   a. `--into <slug>`: add gaps as features/tasks to existing milestone.
   b. Automatic: scan for overlap with in-progress features, add `[ops-gap]` tasks.
   c. Default: group gaps into new `ops-*` milestones (ops-ship-blockers, ops-production-hardening).
5. For new milestones (Mode C): write plan to `/tmp/enterprise-readiness-plan.md`, feed through sdlc-plan.
6. Report: current/target stage, distributed gaps, absorbed tasks, deferred items.

## Key Rules

- Gaps become milestones, not reports.
- Build for the next stage, not three ahead.
- Blast radius drives priority (P0-P3).
"#;

const SDLC_ENTERPRISE_READINESS_SKILL: &str = r#"---
name: sdlc-enterprise-readiness
description: Analyze a project for enterprise readiness and distribute findings into sdlc. Use when assessing production hardening gaps.
---

# SDLC Enterprise Readiness Skill

Use this skill to analyze a project for enterprise readiness and distribute findings into sdlc structure.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Determine current and target stage (MVP/Production/Scale/Enterprise).
2. Load existing sdlc state.
3. Analyze three domains: Storage/Data, Operations, Security.
4. Route findings: add to existing milestone, update active tasks, or create new `ops-*` milestones.
5. For new milestones, write a plan and feed through sdlc-plan.
6. Report: stage assessment, distributed gaps, deferred items.
"#;

pub static SDLC_ENTERPRISE_READINESS: CommandDef = CommandDef {
    slug: "sdlc-enterprise-readiness",
    claude_content: SDLC_ENTERPRISE_READINESS_COMMAND,
    gemini_description:
        "Analyze project for enterprise readiness and distribute findings into sdlc",
    playbook: SDLC_ENTERPRISE_READINESS_PLAYBOOK,
    opencode_description:
        "Analyze project for enterprise readiness and distribute findings into sdlc",
    opencode_hint: "[--stage <stage>] [--into <milestone-slug>]",
    skill: SDLC_ENTERPRISE_READINESS_SKILL,
};
