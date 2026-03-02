use crate::cmd::init::registry::CommandDef;

const SDLC_RUN_COMMAND: &str = r#"---
description: Autonomously drive a feature to completion
argument-hint: <feature-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-run

Drive a feature forward autonomously — executing every action in the state machine loop until the feature is done.

Use `sdlc-next` when you want to execute one step at a time.
Use `sdlc-run` when you want the agent to drive the feature to completion.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

---

## Steps

### 1. Resolve the slug

Get `<feature-slug>` from $ARGUMENTS. If none:
```bash
sdlc next
```
Ask the user which feature to run.

### 2. Confirm scope (if `is_heavy`)

Before starting, show the current phase and what actions will be executed:
```bash
sdlc feature show <slug>
sdlc next --for <slug> --json
```

If any upcoming actions are `is_heavy` (implement_task, fix_review_issues, run_qa), tell the user:
> "This run includes heavy actions (implementation/QA). Proceeding autonomously."

### 3. Run the loop

Repeat until `done`:

```
directive = sdlc next --for <slug> --json

if action == done        → report completion, exit
otherwise                → execute the action (see sdlc-next for action handlers)
                         → loop
```

Execute each action exactly as documented in `sdlc-next`. Do not skip steps or compress multiple actions into one pass — each action advances the state machine and must complete before the next is evaluated.

> **Never call `sdlc feature transition` directly.** Phases advance automatically when artifacts are approved. If a transition isn't happening, an artifact is missing a `draft` or `approve` call — re-check with `sdlc next --for <slug> --json`.

### 4. On unexpected failure

If an action fails in a way that cannot be recovered automatically, stop and report:
1. What action failed
2. What was attempted
3. What the human needs to resolve

Do not loop indefinitely on a failing action.

### 5. On completion

```bash
sdlc feature show <slug>
```

Report the final phase and a summary of everything accomplished.

---

### 6. Next

Always end with a single `**Next:**` line:

| Outcome | Next |
|---|---|
| Feature `done`, milestone has more work | `**Next:** /sdlc-prepare <milestone-slug>` |
| Feature `done`, milestone all released | `**Next:** /sdlc-milestone-uat <milestone-slug>` |
| Feature `done`, no milestone | `**Next:** /sdlc-prepare` |
| Unexpected failure | `**Next:** /sdlc-run <slug>` _(after fixing the blocker)_ |
"#;

const SDLC_RUN_PLAYBOOK: &str = r#"# sdlc-run

Use this playbook to autonomously drive a feature to completion.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Steps

1. Resolve the feature slug. If not provided, run `sdlc next` and pick a feature.
2. Run `sdlc feature show <slug>` and `sdlc next --for <slug> --json` to assess scope.
3. Enter the loop:
   a. Run `sdlc next --for <slug> --json`.
   b. If `action == done` → report completion, exit.
   c. Otherwise → execute the action per sdlc-next protocol, then loop.
4. For each action, execute exactly as documented — write artifacts, implement tasks, run approvals.
5. Never call `sdlc feature transition` directly — phases advance from artifact approvals.
6. On unexpected failure, stop and report what failed and what needs resolving.
"#;

const SDLC_RUN_SKILL: &str = r#"---
name: sdlc-run
description: Autonomously drive a feature to completion. Use when a feature should run unattended through multiple phases.
---

# SDLC Run Skill

Use this skill to autonomously drive a feature through the sdlc state machine to completion.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Workflow

1. Resolve the feature slug.
2. Run `sdlc next --for <slug> --json` to get the current directive.
3. Loop: execute action → re-read directive → repeat.
4. Stop only at `done` or unexpected failure.
5. All actions — including approvals and merge — execute autonomously.
6. Never call `sdlc feature transition` directly; phases advance from artifact approvals.
7. On completion, report what was accomplished and what comes next.
"#;

pub static SDLC_RUN: CommandDef = CommandDef {
    slug: "sdlc-run",
    claude_content: SDLC_RUN_COMMAND,
    gemini_description: "Autonomously drive a feature to completion",
    playbook: SDLC_RUN_PLAYBOOK,
    opencode_description: "Autonomously drive a feature to completion",
    opencode_hint: "<feature-slug>",
    skill: SDLC_RUN_SKILL,
};
