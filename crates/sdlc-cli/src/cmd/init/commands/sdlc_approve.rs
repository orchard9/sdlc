use crate::cmd::init::registry::CommandDef;

const SDLC_APPROVE_COMMAND: &str = r#"---
description: Review and approve an sdlc artifact to advance the feature phase
argument-hint: <feature-slug> <artifact-type>
allowed-tools: Bash, Read
---

# sdlc-approve

Read an artifact, present it for review, and approve it to advance the feature to the next phase.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Usage

```
/sdlc-approve <feature-slug> <artifact-type>
```

Artifact types: `spec` `design` `tasks` `qa_plan` `review` `audit` `qa_results`

## Steps

1. **Resolve args** from $ARGUMENTS. If missing, run `sdlc query needs-approval`.

2. **Read the artifact:**
   ```
   .sdlc/features/<slug>/<type>.md
   ```

3. **Present to user.** Ask: "Approve this [type] for '[slug]'?"

4. **Do NOT approve without explicit user confirmation.**

5. **On approval:**
   ```bash
   sdlc artifact approve <slug> <type>
   ```

6. **On rejection:**
   ```bash
   sdlc artifact reject <slug> <type>
   sdlc comment create <slug> "<feedback>"
   ```

7. **Show what comes next:**
   ```bash
   sdlc next --for <slug>
   ```
"#;

const SDLC_APPROVE_PLAYBOOK: &str = r#"# sdlc-approve

Use this playbook to review and approve an SDLC artifact.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve `<slug>` and `<artifact_type>`.
2. Read the artifact file in `.sdlc/features/<slug>/`:
   - `spec` -> `spec.md`
   - `design` -> `design.md`
   - `tasks` -> `tasks.md`
   - `qa_plan` -> `qa-plan.md`
   - `review` -> `review.md`
   - `audit` -> `audit.md`
   - `qa_results` -> `qa-results.md`
3. Present the artifact for review.
4. Only proceed after explicit user approval.
5. On approval: `sdlc artifact approve <slug> <artifact_type>`.
6. On rejection: `sdlc artifact reject <slug> <artifact_type>` and add a comment.
7. Run `sdlc next --for <slug>` to show what is next.
"#;

const SDLC_APPROVE_SKILL: &str = r#"---
name: sdlc-approve
description: Review and approve an SDLC artifact to advance the feature phase. Use when verifying specs, designs, tasks, reviews, or audits.
---

# SDLC Approve Skill

Use this skill when a user wants to review and approve an SDLC artifact.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Resolve `<slug>` and `<artifact_type>`.
2. Read the artifact under `.sdlc/features/<slug>/`.
3. Present key findings to the user for explicit approval.
4. On approval, run `sdlc artifact approve <slug> <artifact_type>`.
5. On rejection, run `sdlc artifact reject <slug> <artifact_type>` and capture feedback.
6. Run `sdlc next --for <slug>` to show the updated directive.
"#;

pub static SDLC_APPROVE: CommandDef = CommandDef {
    slug: "sdlc-approve",
    claude_content: SDLC_APPROVE_COMMAND,
    gemini_description: "Review and approve an SDLC artifact",
    playbook: SDLC_APPROVE_PLAYBOOK,
    opencode_description: "Review and approve an SDLC artifact",
    opencode_hint: "<feature-slug> <artifact-type>",
    skill: SDLC_APPROVE_SKILL,
};
