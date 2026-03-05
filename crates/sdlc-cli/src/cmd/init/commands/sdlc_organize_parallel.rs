use crate::cmd::init::registry::CommandDef;

const SDLC_ORGANIZE_PARALLEL_COMMAND: &str = r#"---
description: Identify up to 4 parallel work items across active milestones and dispatch them simultaneously
allowed-tools: Bash, Agent
---

# sdlc-organize-parallel

Dispatch up to 4 work items across active milestones simultaneously — one per milestone, max one UAT.
Uses the same selection logic as the dashboard.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Read the parallel work queue

```bash
sdlc parallel-work --json
```

This returns up to 4 items (max 1 UAT slot), sorted by milestone creation order — same as the dashboard's "Current" zone. Each item has:
- `milestone_slug`, `milestone_title`
- `type`: `"feature"` or `"uat"`
- `slug` (feature slug, if type=feature)
- `next_action` (if type=feature)
- `command`: the slash command to run (e.g. `/sdlc-run foo`, `/sdlc-milestone-uat bar`)

### 2. If empty — report and stop

If the array is empty, print:

> No parallel work available. All milestones are done, released, or in the horizon (draft-only features).

**Next:** `/sdlc-status`

### 3. Summarize what will run

Print a table of the slots:

| # | Milestone | Type | Slug / Target | Action | Command |
|---|-----------|------|---------------|--------|---------|
| 1 | … | feature | … | … | `/sdlc-run …` |
| 2 | … | uat | … | — | `/sdlc-milestone-uat …` |

### 4. Dispatch all slots in parallel

Spawn one Agent call per item, all concurrently. Pass the item's `command` field as the prompt.

```
Agent 1: /sdlc-run <slug>            → runs feature to next checkpoint
Agent 2: /sdlc-run <slug>            → runs different milestone's feature
Agent 3: /sdlc-milestone-uat <slug>  → UAT slot (at most one)
Agent 4: /sdlc-run <slug>            → fourth slot if available
```

Do **not** run them sequentially. Use the Agent tool with multiple concurrent calls.

> **409 Conflict:** If a slot returns 409, it is already running — skip it and report
> "(already running)" in the results table. This is normal when re-running quickly.

Wait for all agents to complete.

### 5. Report results

For each completed slot:
- Feature: phase advanced, action taken, outcome.
- UAT: verdict (pass / fail), tasks created if any.

### 6. Next

```
**Next:** `/sdlc-organize-parallel` (run again to advance the next round)
```
"#;

const SDLC_ORGANIZE_PARALLEL_PLAYBOOK: &str = r#"# sdlc-organize-parallel

Dispatch up to 4 parallel work items across active milestones (same logic as dashboard).

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Run `sdlc parallel-work --json` — returns up to 4 items (max 1 UAT), milestone creation order.
2. If empty: report no work available. **Next:** `/sdlc-status`.
3. Print a summary table of all slots.
4. Spawn all items concurrently via Agent — one `/sdlc-run <slug>` or `/sdlc-milestone-uat <slug>` per slot.
5. Wait for all to complete, report results.
6. **Next:** `/sdlc-organize-parallel`
"#;

const SDLC_ORGANIZE_PARALLEL_SKILL: &str = r#"---
name: sdlc-organize-parallel
description: Dispatch up to 4 parallel work items across active milestones simultaneously. Uses the same selection logic as the dashboard Current zone.
---

# SDLC Organize Parallel Skill

Dispatch all active milestone slots concurrently.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Run `sdlc parallel-work --json`. Returns up to 4 items (max 1 UAT), sorted by milestone created_at.
2. If empty: "No parallel work available." **Next:** `/sdlc-status`.
3. Spawn one Agent per item concurrently — `/sdlc-run <slug>` or `/sdlc-milestone-uat <slug>`.
4. Report results. **Next:** `/sdlc-organize-parallel`.
"#;

pub static SDLC_ORGANIZE_PARALLEL: CommandDef = CommandDef {
    slug: "sdlc-organize-parallel",
    claude_content: SDLC_ORGANIZE_PARALLEL_COMMAND,
    gemini_description: "Identify up to 4 parallel work items across active milestones and dispatch them simultaneously",
    playbook: SDLC_ORGANIZE_PARALLEL_PLAYBOOK,
    opencode_description: "Identify up to 4 parallel work items and dispatch them simultaneously",
    opencode_hint: "",
    skill: SDLC_ORGANIZE_PARALLEL_SKILL,
};
