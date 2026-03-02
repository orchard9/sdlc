use crate::cmd::init::registry::CommandDef;

const SDLC_HYPOTHETICAL_DO_COMMAND: &str = r#"---
description: Execute a READY hypothetical plan — gate-check the verdict, orient on architecture, execute in component order, verify every file, log completion
argument-hint: <slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-hypothetical-do

Execute the hypothetical plan at `.sdlc/hypotheticals/<slug>/`. This command is execution-only — the plan was decided during `/sdlc-hypothetical-planning`. Do not redesign; execute and verify.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Gate check

Read `.sdlc/hypotheticals/<slug>/manifest.yaml`. If `status != ready`, print the confidence verdict and blockers, then stop with: `**Blocked:** /sdlc-hypothetical-planning <slug>`

### 2. Orient — read ALL artifacts before touching any file

1. `confidence.md` — implementation notes: highest-risk file, hardest problem, silent-failure risks
2. `architecture.md` — system-level shape, what changes and what doesn't
3. `components.md` — per-component breakdown, execution order (inside-out)
4. `file-manifest.md` — the contract: Added, Modified, Removed, Unchanged

State out loud before proceeding:
- "Execution order: [components in dependency order]"
- "Highest-risk file: [file] because [reason]"
- "Silent-failure risk: [risk]"

### 3. Execute in order

Order: core/foundation components first → consumers → edges. Within manifest: Added first, Modified second, Removed last (only after replacements are written).

For each file:
- **Added:** create at exact path, implement what `components.md` specifies
- **Modified:** read existing file first, apply only the listed changes
- **Removed:** verify replacement is in place, then delete

### 4. Step back before declaring done

- Go through the manifest line by line: Added exists? Modified has the change? Removed is gone?
- Did I write anything not in the manifest? If structural: halt and flag. If minor implied (import, barrel): log it.
- Did I apply every safeguard from `confidence.md` implementation notes?
- Run build/type-check if available.

### 5. Verify — manifest check

Print an explicit check for every listed file. Fix any failures before proceeding.

### 6. Complete

Write `.sdlc/hypotheticals/<slug>/execution-log.md`: timestamp, files changed, deviations (if any), safeguards applied, build result.

Update `manifest.yaml` status to `completed`.

### 7. Report

```
✓ <subject>
  Added: N | Modified: N | Removed: N
  Deviations: N (all logged)
  Build: passed
```

**Next:** commit changes and continue with the next step in the workflow.
"#;

const SDLC_HYPOTHETICAL_DO_PLAYBOOK: &str = r#"# sdlc-hypothetical-do

Execute a READY hypothetical plan from `.sdlc/hypotheticals/<slug>/`.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Read `manifest.yaml`. If `status != ready`, print blockers and stop: `**Blocked:** /sdlc-hypothetical-planning <slug>`
2. Read ALL artifacts before touching files: `confidence.md` (implementation notes) → `architecture.md` → `components.md` → `file-manifest.md`.
3. State execution order (from components.md, inside-out), highest-risk file, and silent-failure risk before starting.
4. Execute: Added first (create), Modified second (read-then-change), Removed last (after replacements written).
5. Step back: line-by-line manifest check. Any unplanned structural changes? Halt and re-plan. Minor implied additions? Log them.
6. Apply all safeguards from `confidence.md` implementation notes.
7. Run build/type-check if available. Fix failures before completing.
8. Write `execution-log.md` (timestamp, changed files, deviations, build result). Update `manifest.yaml` to `completed`.
9. End: `**Next:** commit changes`
"#;

const SDLC_HYPOTHETICAL_DO_SKILL: &str = r#"---
name: sdlc-hypothetical-do
description: Execute a READY hypothetical plan — gate-check verdict, orient on architecture, execute in component order, verify every file, log completion. Use after /sdlc-hypothetical-planning returns READY.
---

# SDLC Hypothetical Do Skill

Execute the plan; do not redesign it.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Read `manifest.yaml`. Abort if `status != ready`. Print blockers from `confidence.md` and end with `/sdlc-hypothetical-planning <slug>`.
2. Read all artifacts: `confidence.md`, `architecture.md`, `components.md`, `file-manifest.md`. State execution order and risks before writing any file.
3. Execute inside-out (core → consumers → edges). Added first, Modified second, Removed last.
4. Step back: explicit manifest check (every listed file), deviation audit, safeguards applied.
5. Run build/type-check. Fix failures.
6. Write `execution-log.md`, update `manifest.yaml` to `completed`.
7. End: `**Next:** commit changes`
"#;

pub static SDLC_HYPOTHETICAL_DO: CommandDef = CommandDef {
    slug: "sdlc-hypothetical-do",
    claude_content: SDLC_HYPOTHETICAL_DO_COMMAND,
    gemini_description: "Execute a READY hypothetical plan — gate-check, orient, execute in order, verify, log completion",
    playbook: SDLC_HYPOTHETICAL_DO_PLAYBOOK,
    opencode_description: "Execute a READY hypothetical plan — gate-check, orient, execute in order, verify, log completion",
    opencode_hint: "<slug>",
    skill: SDLC_HYPOTHETICAL_DO_SKILL,
};
