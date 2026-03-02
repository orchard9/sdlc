use crate::cmd::init::registry::CommandDef;

const SDLC_SKILL_BUILD_COMMAND: &str = r#"---
description: Build a project-aware skill for a specific domain or capability
argument-hint: <slug> "<purpose>"
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-skill-build

Build a single `.claude/skills/<slug>/SKILL.md` grounded in this project's actual tech stack,
conventions, file paths, and quality commands.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Parse arguments

`$ARGUMENTS` is `<slug> "<purpose>"`.

- `<slug>` — kebab-case skill name, e.g. `rust-api`, `react-components`, `db-migrations`
- `<purpose>` — one sentence describing what this skill covers

### 2. Read the project

Collect the context needed to write grounded content:

```bash
sdlc state
```

Then read (if present):
- `CLAUDE.md` — conventions, key files, coding rules
- Root config files (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`)
- Key source directories relevant to the skill's domain

Read existing skills to match conventions:

```bash
ls .claude/skills/
```

Read one existing skill for reference format if any exist.

### 3. Write `.claude/skills/<slug>/SKILL.md`

Use the standard skill format. Every section must use **real values from this project**:

```markdown
---
name: <slug>
description: Use when <specific triggers from this project's domain>. Examples — "<real example 1>", "<real example 2>".
---

# <Role Title>

<One sentence: what this skill does and which layer of the project it owns.>

## Principles

1. **<Principle from CLAUDE.md or codebase conventions>.** <Why it matters here>.
2. **<Principle>.** <Explanation>.
(3–5 principles grounded in this project's actual rules)

## Quick Reference

| Area | Path | Notes |
|------|------|-------|
| <area> | `<actual/path/from/project>` | <note> |
(4–8 rows with real paths)

## Phase 1: Understand the Change

Before writing any code, read:
1. `<real file path>` — <what to learn from it>
2. `<real file path>` — <what to learn from it>
3. <real test pattern or integration test location>

State: what is being added/changed and which layer it touches.

## Phase 2: Design the Interface

<Domain-specific: types, API signatures, contracts — grounded in project's actual patterns>

## Phase 3: Implement

Work in this order:
1. <project-specific implementation step>
2. <project-specific step>
(steps that reflect actual project structure and conventions from CLAUDE.md)

## Step Back: Challenge Before Committing

Before finalizing, ask:

### 1. <Relevant challenge for this domain>
> "<Challenge prompt>"
- <specific constraint from this project>

### 2. <Second challenge>
> "<Challenge prompt>"
- <specific constraint>

## Phase 4: Verify

```bash
<actual quality commands from .sdlc/config.yaml or CLAUDE.md>
```

## Done Gate

- [ ] <Project-specific criterion>
- [ ] <Project-specific criterion>
- [ ] All quality gates pass (`sdlc quality check` or equivalent)
- [ ] Changes committed
```

### 4. Commit

```bash
git add .claude/skills/<slug>/
git commit -m "feat: add <slug> skill"
```

**Next:** invoke the skill directly — `Use <slug> skill to <first task>`
"#;

const SDLC_SKILL_BUILD_PLAYBOOK: &str = r#"# sdlc-skill-build

Build a project-aware skill for a specific domain or capability.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Parse `$ARGUMENTS` as `<slug> "<purpose>"`.
2. Read project context: `sdlc state`, `CLAUDE.md`, root config files, relevant source dirs.
3. Read existing `.claude/skills/` for format conventions.
4. Write `.claude/skills/<slug>/SKILL.md` — every section uses real paths and commands from this project:
   - `name`, `description` with trigger examples
   - Principles grounded in CLAUDE.md/codebase conventions
   - Quick Reference table with actual file paths
   - Phase 1–4 workflow with real files and real quality commands
   - Step Back challenges specific to this domain
   - Done Gate checklist
5. Commit: `git add .claude/skills/<slug>/ && git commit -m "feat: add <slug> skill"`

## Key Rules

- Every path in Quick Reference must be a real path from the project.
- Quality commands in Phase 4 must match the project's actual stack.
- No placeholder text in the final file — if you don't know a value, read more code.
- Fire and iterate — write the skill, don't gate on approval.
"#;

const SDLC_SKILL_BUILD_SKILL: &str = r#"---
name: sdlc-skill-build
description: Build a project-aware skill for a specific domain. Use when adding a new skill grounded in this project's actual stack, conventions, and file paths.
---

# SDLC Skill-Build Skill

Build a single `.claude/skills/<slug>/SKILL.md` grounded in this project's actual codebase.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Parse args: `<slug> "<purpose>"`.
2. Read project: `sdlc state`, `CLAUDE.md`, root config files, relevant source dirs.
3. Read existing `.claude/skills/` for format conventions.
4. Write `.claude/skills/<slug>/SKILL.md` with:
   - Principles from actual project conventions
   - Quick Reference with real file paths
   - Phase 1–4 workflow with real files and real quality commands
   - Step Back challenges for this domain
   - Done Gate checklist
5. Commit the new skill.

## Key Rules

- Every path must be real. Every command must work. No placeholders.
- Fire and iterate — no approval gate.
"#;

pub static SDLC_SKILL_BUILD: CommandDef = CommandDef {
    slug: "sdlc-skill-build",
    claude_content: SDLC_SKILL_BUILD_COMMAND,
    gemini_description: "Build a project-aware skill for a specific domain or capability",
    playbook: SDLC_SKILL_BUILD_PLAYBOOK,
    opencode_description: "Build a project-aware skill for a specific domain or capability",
    opencode_hint: r#"<slug> "<purpose>""#,
    skill: SDLC_SKILL_BUILD_SKILL,
};
