use crate::cmd::init::registry::CommandDef;

const SDLC_RECAP_COMMAND: &str = r#"---
description: Produce a state-aware session recap with forward motion — synthesize progress, classify remaining work, and create sdlc artifacts so no session ends without a concrete next step
argument-hint: [feature-slug | milestone-slug]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-recap

Produce a state-aware session recap. Read actual sdlc state — not just conversation context — synthesize what was accomplished, classify remaining work into action categories, create concrete forward artifacts, and end with exactly one `**Next:**` line.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

---

## Step 1 — Gather state

```bash
sdlc status --json
git log --oneline -20
git diff --stat HEAD~5
```

If a milestone or feature slug is provided in `$ARGUMENTS`:
```bash
sdlc milestone info <slug> --json     # if slug is a milestone
sdlc feature show <slug>              # if slug is a feature
sdlc task list <slug>                 # pending tasks
```

Also read recent escalations, active feature phases, and recent git commit messages to understand what was in progress.

---

## Step 2 — Synthesize

Produce the following four sections:

### Working On

State the session goal clearly in one sentence. What was the agent trying to accomplish?

### Completed

List concrete deliverables:
- Features advanced (slug, from-phase → to-phase)
- Artifacts written (file paths)
- Tasks completed (feature#id)
- Commits landed (short hash + message)

### Remaining

For each unresolved item, classify it:

| Classification | Criteria |
|---|---|
| **Fixable** | Concrete, bounded, doable in 1-2 sessions with no external decisions needed |
| **Needs input** | Requires a human decision, external information, or approval before proceeding |
| **Complex** | Multi-session, uncertain scope, or needs design thinking before acting |

List each item under its classification with a one-sentence description.

### Forward Motion

For each item, take the action immediately:

**For Complex items:**
```bash
sdlc ponder create "<problem-as-question>" --brief "<one-paragraph context>"
```

**For Needs input items:**
```bash
sdlc task add <feature-slug> "[escalate] <description of what decision is needed>"
```

**For Fixable items:**
```bash
sdlc task add <feature-slug> "<description>"
```

If no slug is obvious for a task, use the feature closest to the work described.

---

## Step 3 — Commit completed work

Stage and commit any artifacts, results, or code written during the session:

```bash
git add -A
git commit -m "session: <brief summary of what was accomplished>"
```

If nothing was written (clean read-only session), skip the commit.

---

## Step 4 — Output and next step

**Next-line rules (pick exactly one):**

| Situation | Next line |
|---|---|
| Ponder sessions created | `**Next:** /sdlc-ponder <first-ponder-slug>` |
| Only tasks created, milestone active | `**Next:** /sdlc-run <first-task-feature-slug>` |
| Only tasks created, no milestone | `**Next:** /sdlc-status` |
| Everything complete, nothing remaining | `**Next:** /sdlc-milestone-uat <milestone-slug>` (if milestone) or `/sdlc-status` |

Always end output with exactly one `**Next:**` line. No exceptions.
"#;

const SDLC_RECAP_PLAYBOOK: &str = r#"# sdlc-recap

Produce a state-aware session recap and create forward motion artifacts for all remaining work.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. **Gather state** — run `sdlc status --json`, `git log --oneline -20`, `git diff --stat HEAD~5`. If a slug is provided, also read `sdlc milestone info <slug> --json` or `sdlc feature show <slug>`.
2. **Synthesize** — produce four sections: Working On (goal), Completed (deliverables with paths/IDs), Remaining (classified), Forward Motion (actions taken).
3. **Classify remaining work:**
   - Fixable → `sdlc task add <feature> "<description>"`
   - Needs input → `sdlc task add <feature> "[escalate] <decision needed>"`
   - Complex → `sdlc ponder create "<question>" --brief "<context>"`
4. **Commit completed work** — `git add -A && git commit -m "session: <summary>"` (skip if nothing written).
5. **End with exactly one `**Next:**` line:**
   - Ponder created → `/sdlc-ponder <first-slug>`
   - Tasks created → `/sdlc-run <feature-slug>`
   - Everything done → `/sdlc-milestone-uat <milestone>` or `/sdlc-status`
"#;

const SDLC_RECAP_SKILL: &str = r#"---
name: sdlc-recap
description: Produce a state-aware session recap with forward motion. Reads actual sdlc state to synthesize progress, classify remaining work, and create tasks or ponder entries. Use after UAT failure, at session end, or when handing off between agents.
---

# SDLC Recap Skill

Produce a session recap grounded in actual sdlc state — not just conversation context.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Read state: `sdlc status --json`, `git log --oneline -20`, `git diff --stat HEAD~5`. If a slug was provided, also read feature/milestone info and task list.
2. Synthesize four sections: **Working On** (goal), **Completed** (deliverables), **Remaining** (with classification), **Forward Motion** (actions taken).
3. Classify each remaining item as Fixable / Needs input / Complex and take the action immediately: `sdlc task add` or `sdlc ponder create`.
4. Commit completed work: `git add -A && git commit -m "session: <summary>"`.
5. End with exactly one `**Next:**` line: ponder sessions created → `/sdlc-ponder <slug>`; tasks created → `/sdlc-run <slug>`; done → `/sdlc-status`.
"#;

pub static SDLC_RECAP: CommandDef = CommandDef {
    slug: "sdlc-recap",
    claude_content: SDLC_RECAP_COMMAND,
    gemini_description: "Produce a state-aware session recap with forward motion",
    playbook: SDLC_RECAP_PLAYBOOK,
    opencode_description: "Produce a state-aware session recap with forward motion",
    opencode_hint: "[feature-slug | milestone-slug]",
    skill: SDLC_RECAP_SKILL,
};
