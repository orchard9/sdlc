use crate::cmd::init::registry::CommandDef;

const SDLC_SPECIALIZE_COMMAND: &str = r#"---
description: Survey this project and generate a tailored AI team (Claude agents + skills)
argument-hint: [project-description]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-specialize

Generate a project-specific AI team — Claude agent personas and blueprint-style skills —
tailored to this project's tech stack, domain, and team roles. Runs across 4 sessions with
explicit human checkpoints so you approve each stage before files are written.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Overview

This command produces:
- **`.claude/agents/<first-last>.md`** — persona agents with YAML frontmatter, background,
  Principles, This Codebase, ALWAYS/NEVER rules, and When Stuck section
- **`.claude/skills/<domain-role>/SKILL.md`** — blueprint skills with Quick Reference table,
  Phase 1–4 workflow, Step Back challenges, and Done Gate checklist

---

## Session 1: Survey the Project

Read the project to understand its domain, tech stack, and current state:

```bash
sdlc state
```

Then read (if present):
- `VISION.md` or `docs/vision.md`
- `docs/architecture.md` or `ARCHITECTURE.md`
- `AGENTS.md` or `CLAUDE.md`
- `README.md`
- Root config files (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, etc.)
- Key source directories (list top-level dirs)

Summarize:
1. **Project purpose** — what does it do in one sentence?
2. **Tech stack** — languages, frameworks, key libraries
3. **Domain areas** — e.g., "CLI tooling", "API layer", "frontend", "data pipeline"
4. **Current SDLC phase** — active features, milestones, maturity

### ✋ Gate 1: Confirm Understanding

Present your summary to the user:

> "Here's what I found about [project]:
> - Purpose: ...
> - Stack: ...
> - Domain areas: ...
> - Current state: ...
>
> Does this look right before I design the team?"

**Wait for explicit user confirmation before proceeding.**

---

## Session 2: Design the Team Roster

Design 3–5 specialist roles that match the project's actual domain areas.

For each role, define:
- **Persona name** (first-last, e.g., `alex-chen`) — a real-sounding human name
- **Role title** — e.g., "API Engineer", "Frontend Builder", "Data Pipeline Architect"
- **Domain ownership** — which files/directories/subsystems they own
- **Model assignment** — `opus` for architects/heavy reasoners, `sonnet` for implementers
- **Color** — pick from: `orange`, `blue`, `green`, `purple`, `red`, `yellow`, `cyan`
- **Skill name** — kebab-case domain slug, e.g., `api-engineer`, `frontend-builder`

Present as a table:

| Name | Role | Domain | Model | Color | Skill |
|------|------|--------|-------|-------|-------|
| alex-chen | API Engineer | `src/api/`, `src/routes/` | sonnet | blue | api-engineer |
| ... | ... | ... | ... | ... | ... |

### ✋ Gate 2: Approve the Roster

> "Here's the proposed team roster for [project]. Does this look right?
> Any roles to add, remove, or rename before I generate the files?"

**Wait for explicit user approval. Do NOT write any files before this gate.**

---

## Session 3: Generate Agents and Skills

For each approved roster entry, generate two files.

### Agent format: `.claude/agents/<first-last>.md`

```markdown
---
name: <first-last>
description: Use when <specific domain triggers>. Examples — "<example 1>", "<example 2>", "<example 3>".
model: <opus|sonnet|haiku>
color: <color>
---

You are <Full Name>, <background paragraph — 3-4 sentences describing their career history at named companies/projects, their area of expertise, and their deeply held technical philosophy. Be specific and concrete, not generic>.

## Your Principles

- **<Principle name>.** <One sentence explanation of why this matters in this codebase>.
- **<Principle name>.** <One sentence explanation>.
- (3–5 principles total)

## This Codebase

**<Area 1>:**
- `path/to/file.ext` — brief description of what it does
- `path/to/dir/` — brief description

**<Area 2>:**
- `path/to/file.ext` — brief description
(cover 2–4 domain areas with the actual file paths from the project)

## ALWAYS

- <concrete, specific rule about this codebase — not generic advice>
- <specific rule>
- (3–6 rules)

## NEVER

- <concrete anti-pattern specific to this domain>
- <specific anti-pattern>
- (3–6 rules)

## When You're Stuck

1. **<Common failure mode>:** <Specific debugging approach with actual commands or file paths>.
2. **<Common failure mode>:** <Specific approach>.
3. (2–4 entries)
```

### Skill format: `.claude/skills/<domain-role>/SKILL.md`

```markdown
---
name: <domain-role>
description: Use when <triggers>. Delegate to **<Full Name>** for implementation.
---

# <Role Title>

You are a <domain> specialist. Delegate to **<Full Name>** for implementation.

## Principles

1. **<Principle>.** <Explanation>.
2. **<Principle>.** <Explanation>.
(3–5 principles)

## Quick Reference

| Area | Path | Notes |
|------|------|-------|
| <area> | `<actual/path>` | <note> |
(use real paths from the project)

## Phase 1: Understand the Change

Before writing any code, read:
1. <specific files relevant to this domain>
2. <related interface/type files>
3. <test patterns in use>

State: what is being added/changed and which layer it lives in.

## Phase 2: Design the Interface

<Domain-specific interface design guidance — types, APIs, contracts>

## Phase 3: Implement

Delegate to **<Full Name>** for the implementation. Work in this order:
1. <domain-specific implementation order>
2. ...

## Step Back: Challenge Before Committing

Before finalizing the implementation, ask:

### 1. <First challenge question for this domain>
> "<Challenge prompt>"
- <specific constraint to check>

### 2. <Second challenge question>
> "<Challenge prompt>"
- <specific constraint to check>

(2–4 challenges relevant to this domain)

## Phase 4: Verify

```bash
<actual quality commands for this project's stack>
```

## Done Gate

- [ ] <Specific completion criterion for this domain>
- [ ] <Specific criterion>
- [ ] All tests pass
- [ ] <Stack-specific quality check passes>
```

Write all agents to `.claude/agents/` and all skills to `.claude/skills/`.

---

## Session 4: Update AGENTS.md

Add a `## Team` section to `AGENTS.md` (or update if it exists) listing each agent and their domain:

```markdown
## Team

| Agent | Role | Domain | Invoke When |
|-------|------|--------|-------------|
| @<first-last> | <Role> | <Domain> | <When to use> |
```

### ✋ Gate 4: Final Confirmation

List all files created:
```
Created:
  .claude/agents/alex-chen.md
  .claude/agents/...
  .claude/skills/api-engineer/SKILL.md
  .claude/skills/.../SKILL.md
  AGENTS.md (updated Team section)
```

> "All done. Your project now has a tailored AI team. Use `/sdlc-next` to drive features forward
> with these specialists, or invoke agents directly by name in Claude Code."
"#;

const SDLC_SPECIALIZE_PLAYBOOK: &str = r#"# sdlc-specialize

Survey this project and generate a tailored AI team (agents + skills).

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Read project files: `VISION.md`, `AGENTS.md`, `CLAUDE.md`, root config files, key source dirs.
2. Run `sdlc state` to understand current SDLC phase and maturity.
3. Summarize: purpose, tech stack, domain areas, current state.
4. **Gate 1**: Present summary to user — wait for confirmation.
5. Design 3-5 specialist roles matching domain areas (persona name, role, domain ownership, model, skill name).
6. **Gate 2**: Present roster table — wait for user approval.
7. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each role.
8. Update `AGENTS.md` with a Team section listing all agents.
9. **Gate 3**: List all created files — confirm completion.

## Key Rules

- Always include user perspectives (not just engineers).
- Agents get concrete `This Codebase` sections with real file paths.
- Skills get 4-phase workflow (Understand, Design, Implement, Verify) + Done Gate.
- Wait for explicit user approval at each gate before proceeding.
"#;

const SDLC_SPECIALIZE_SKILL: &str = r#"---
name: sdlc-specialize
description: Survey this project and generate a tailored AI team (agents + skills). Use when setting up project-specific agent personas and domain skills.
---

# SDLC Specialize Skill

Use this skill to generate a project-specific AI team with agent personas and skills.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Survey: read project config, source dirs, VISION.md, AGENTS.md, `sdlc state`.
2. Summarize purpose, tech stack, domain areas, current state.
3. Gate 1: confirm understanding with user.
4. Design 3-5 specialist roles matching domain areas.
5. Gate 2: approve roster with user.
6. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each.
7. Update AGENTS.md with Team section.
8. Gate 3: confirm all files created.

## Key Rules

- Agents have: frontmatter, background, Principles, This Codebase, ALWAYS/NEVER, When Stuck.
- Skills have: Quick Reference, Phase 1-4, Step Back, Done Gate.
- Always wait for user approval at each gate.
"#;

pub static SDLC_SPECIALIZE: CommandDef = CommandDef {
    slug: "sdlc-specialize",
    claude_content: SDLC_SPECIALIZE_COMMAND,
    gemini_description: "Survey this project and generate a tailored AI team (agents + skills)",
    playbook: SDLC_SPECIALIZE_PLAYBOOK,
    opencode_description: "Survey this project and generate a tailored AI team (agents + skills)",
    opencode_hint: "[project-description]",
    skill: SDLC_SPECIALIZE_SKILL,
};
