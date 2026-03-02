use crate::cmd::init::registry::CommandDef;

const SDLC_STATUS_COMMAND: &str = r#"---
description: Show SDLC state for the project or a specific feature
argument-hint: [feature-slug]
allowed-tools: Bash
---

# sdlc-status

Show the current SDLC state — what features exist, what phase they're in, and what needs attention.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Usage

```
/sdlc-status              → project-wide overview
/sdlc-status <slug>       → detailed view of one feature
```

## Project-wide overview

```bash
sdlc state
sdlc next
sdlc query needs-approval
sdlc query blocked
sdlc query ready
sdlc ponder list
```

Show features grouped by: needs approval, blocked, in progress, ready. Include active ponder entries (exploring/converging ideas).

## Single-feature detail

```bash
sdlc feature show <slug>
sdlc next --for <slug>
sdlc task list <slug>
sdlc comment list <slug>
```

Show phase, artifact status, open tasks, comments, and the next action.

## Lifecycle

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```
"#;

const SDLC_STATUS_PLAYBOOK: &str = r#"# sdlc-status

Use this playbook to report SDLC state for the whole project or one feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Project view

Run:
- `sdlc state`
- `sdlc next`
- `sdlc query needs-approval`
- `sdlc query blocked`
- `sdlc query ready`
- `sdlc ponder list`

## Feature view

Run:
- `sdlc feature show <slug>`
- `sdlc next --for <slug>`
- `sdlc task list <slug>`
- `sdlc comment list <slug>`
"#;

const SDLC_STATUS_SKILL: &str = r#"---
name: sdlc-status
description: Show SDLC state for the project or a specific feature. Use when checking progress, blockers, or next actions.
---

# SDLC Status Skill

Use this skill when a user asks for SDLC status across the project or for one feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

Project view:
- `sdlc state`
- `sdlc next`
- `sdlc query needs-approval`
- `sdlc query blocked`
- `sdlc query ready`
- `sdlc ponder list`

Feature view:
- `sdlc feature show <slug>`
- `sdlc next --for <slug>`
- `sdlc task list <slug>`
- `sdlc comment list <slug>`
"#;

pub static SDLC_STATUS: CommandDef = CommandDef {
    slug: "sdlc-status",
    claude_content: SDLC_STATUS_COMMAND,
    gemini_description: "Show SDLC state for the project or a feature",
    playbook: SDLC_STATUS_PLAYBOOK,
    opencode_description: "Show SDLC state for the project or a specific feature",
    opencode_hint: "[feature-slug]",
    skill: SDLC_STATUS_SKILL,
};
