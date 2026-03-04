use crate::cmd::init::registry::CommandDef;

const SDLC_NEXT_COMMAND: &str = r#"---
description: Get the next directive for a feature and act on it
argument-hint: <feature-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-next

Read the next directive for a feature and act on it. This is the primary entry point for driving features forward.

## What is sdlc?

`sdlc` is a project management state machine. It tracks features through a lifecycle:

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

Every phase requires specific Markdown artifacts to be written and approved before advancing.
`sdlc next --json` tells you exactly what to do next. You act on it, submit the artifact, and the phase advances.

## Steps

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

### 1. Resolve the slug

Get `<feature-slug>` from $ARGUMENTS. If none provided:
```bash
sdlc next
```
Show the output and ask the user which feature to drive.

### 2. Get the directive

```bash
sdlc next --for <slug> --json
```

Key fields: `action`, `message`, `output_path`, `current_phase`, `is_heavy`, `gates`.

### 3. Handle `done`

> "All SDLC phases complete for '[slug]'."

### 4. Execute the directive

For **artifact creation** (`create_spec`, `create_design`, `create_tasks`, `create_qa_plan`, `create_review`, `create_audit`):
1. Run `sdlc feature show <slug> --json` for context
2. Read existing artifacts in `.sdlc/features/<slug>/`
3. Write a thorough Markdown artifact to `output_path`

For `create_design` on a **UI feature**, also write `mockup.html` in the same directory:
- Single self-contained file — inline `<style>` and `<script>`, no CDN or external resources
- Valid HTML5 (`<!DOCTYPE html>`) with a `<nav>` or tab bar to navigate between screens
- Named sections (`<section id="screen-*">`) for each major UI state
- Representative colors and typography; pixel-perfect fidelity is not required
- Reference it from `design.md` with a relative link: `[Mockup](mockup.html)`

For non-UI features (backend, CLI, config-only): `mockup.html` is optional; ASCII wireframes in `design.md` are sufficient.

For **approval** (`approve_spec`, `approve_design`, `approve_tasks`, `approve_qa_plan`, `approve_merge`):
1. Read the artifact at `output_path`, verify it is complete and correct
2. Run `sdlc artifact approve <slug> <artifact_type>` autonomously — no confirmation needed

For **approve_review** and **approve_audit**:
1. Read the artifact and enumerate every finding
2. For each finding, take exactly one action before approving:
   - **Fix now** — implement a targeted code change (not a broad `/fix-all` or `/remediate` sweep)
   - **Track** — `sdlc task add <slug> "finding: <summary>"` to address in a future cycle
   - **Accept** — document why no action is needed (e.g. "operator-controlled, no external exposure")
3. No finding may be silently skipped — every one must be explicitly resolved
4. `sdlc artifact approve <slug> <artifact_type>` only after all findings are accounted for

For **implementation** (`implement_task`):
1. Run `sdlc task list <slug>` to find the next pending task
2. Read design and tasks artifacts for context
3. Implement the task, then run `sdlc task complete <slug> <task-id>`

For **merge** (`merge`):
```bash
sdlc merge <slug> --json
```
This transitions the feature to `released`. Execute immediately — no confirmation needed.

For **gates** (`wait_for_approval`, `unblock_dependency`):
Stop and report clearly. These require human intervention before the feature can advance.

### 5. Show updated state

```bash
sdlc next --for <slug>
```
"#;

const SDLC_NEXT_PLAYBOOK: &str = r#"# sdlc-next

Use this playbook to drive the next SDLC directive for a feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> **NEVER edit `.sdlc/` YAML files directly.** All state changes go through `sdlc` CLI commands. The only files you write are Markdown artifacts to the `output_path` from `sdlc next --json`.

## Steps

1. Resolve the slug.
   - If one is not provided, run `sdlc next` and pick a feature.
2. Run `sdlc next --for <slug> --json`.
3. Parse directive fields: `action`, `message`, `output_path`, `current_phase`, `is_heavy`, `gates`.
4. For creation actions:
   - Read feature context and existing artifacts.
   - Write the required artifact to `output_path`.
   - Mark it draft with `sdlc artifact draft <slug> <artifact_type>`.
   - For `create_design` on a UI feature: also write `mockup.html` (self-contained, inline CSS/JS, named `<section id="screen-*">` blocks, navigation bar). Reference it from `design.md`: `[Mockup](mockup.html)`.
5. For approval actions (`approve_spec`, `approve_design`, `approve_tasks`, `approve_qa_plan`, `approve_merge`):
   - Read the artifact at `output_path`, verify it is complete and correct.
   - Run `sdlc artifact approve <slug> <artifact_type>` autonomously.
5a. For `approve_review` and `approve_audit`: enumerate every finding. Fix now (targeted), track (`sdlc task add`), or accept (document rationale). No silent skips. Approve only after all findings are resolved.
6. For implementation:
   - Run `sdlc task list <slug>`.
   - Implement the next task and run `sdlc task complete <slug> <task_id>`.
7. Run `sdlc next --for <slug>` to show updated state.
"#;

const SDLC_NEXT_SKILL: &str = r#"---
name: sdlc-next
description: Get the next SDLC directive for a feature and act on it. Use when driving a feature forward one step at a time.
---

# SDLC Next Skill

Use this skill when a user asks for the next SDLC action for a feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Workflow

1. Resolve the feature slug.
2. Run `sdlc next --for <slug> --json`.
3. Follow the directive fields (`action`, `message`, `output_path`, `gates`).
4. For approval or dependency gates, surface context and wait for explicit user approval.
5. For creation actions, write the requested artifact at `output_path`.
   For `create_design` on a UI feature: also produce `mockup.html` — self-contained HTML5 (no external resources), with a nav bar and named `<section id="screen-*">` blocks for each UI state. Reference it from `design.md`: `[Mockup](mockup.html)`.
6. For implementation actions, complete the next pending task.
7. Run `sdlc next --for <slug>` to confirm what comes next.
"#;

pub static SDLC_NEXT: CommandDef = CommandDef {
    slug: "sdlc-next",
    claude_content: SDLC_NEXT_COMMAND,
    gemini_description: "Get the next SDLC directive for a feature and act on it",
    playbook: SDLC_NEXT_PLAYBOOK,
    opencode_description: "Get the next SDLC directive for a feature and act on it",
    opencode_hint: "<feature-slug>",
    skill: SDLC_NEXT_SKILL,
};
