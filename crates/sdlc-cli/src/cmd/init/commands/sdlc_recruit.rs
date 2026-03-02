use crate::cmd::init::registry::CommandDef;

const SDLC_RECRUIT_COMMAND: &str = r#"---
description: Recruit an expert thought partner — creates an agent with real background, strong opinions, and domain expertise
argument-hint: <role description or domain context>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
---

# sdlc-recruit

Identify and recruit the ideal expert for a specific need. Produces a fully realized
agent definition — not a generic role, but a specific person with career history,
technical philosophy, and strong opinions.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Steps

### 1. Orient

Read the project to understand what expertise is needed:
- `VISION.md` or `docs/vision.md`
- `CLAUDE.md` or `AGENTS.md`
- Root config files for tech stack signals
- `sdlc state` for current project context

Parse $ARGUMENTS for the domain/role being recruited for.

### 2. Design the expert

Create a specific person, not a generic role:
- **Real name** (first-last, e.g., `kai-tanaka`)
- **Career background** — 3-4 sentences at named companies/projects with concrete
  technical contributions
- **Technical philosophy** — deeply held beliefs that create productive tension
- **Strong opinions** — specific to this domain, not generic best practices
- **Blind spots** — what this expert might miss (so other partners compensate)

### 3. Create the agent

Write to `.claude/agents/<name>.md`:

```markdown
---
name: <first-last>
description: Use when <specific triggers>. Examples — "<example 1>", "<example 2>".
model: opus
---

You are <Full Name>, <career background paragraph>.

## Your Principles
- **<Principle>.** <Why this matters>.
(3-5 principles)

## This Codebase
**<Area>:**
- `path/to/file` — relevance
(actual paths from the project)

## ALWAYS
- <concrete rule about this codebase>
(3-6 rules)

## NEVER
- <concrete anti-pattern for this domain>
(3-6 rules)

## When You're Stuck
1. **<Failure mode>:** <Specific approach with actual commands/paths>.
(2-4 entries)
```

### 4. Optionally register with a ponder entry

If recruiting for a ponder session:
```bash
sdlc ponder team add <ponder-slug> --name "<name>" --role "<role>" \
  --context "<why this person>" --agent .claude/agents/<name>.md
```

---

### 5. Next

| Context | Next |
|---|---|
| Within a ponder session | `**Next:** /sdlc-ponder <slug>` (continue with new partner) |
| For a pressure test | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Standalone | `**Next:** Use @<name> in conversation to invoke the agent` |
"#;

const SDLC_RECRUIT_PLAYBOOK: &str = r#"# sdlc-recruit

Recruit an expert thought partner as a persistent agent.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Orient: read project context (CLAUDE.md, AGENTS.md, sdlc state).
2. Design the expert: real name, career background at named companies, strong opinions, blind spots.
3. Write agent to `.claude/agents/<name>.md` with principles, codebase context, ALWAYS/NEVER rules.
4. Optionally register with ponder entry:
   `sdlc ponder team add <slug> --name "<name>" --role "<role>" --context "<why>" --agent .claude/agents/<name>.md`
5. **Next:** use @<name> in conversation, or continue ponder session.
"#;

const SDLC_RECRUIT_SKILL: &str = r#"---
name: sdlc-recruit
description: Recruit an expert thought partner as a persistent agent. Use when you need domain expertise, user perspectives, or productive skepticism.
---

# SDLC Recruit Skill

Use this skill to recruit an expert agent for the project.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Orient: read project context (CLAUDE.md, AGENTS.md, stack signals).
2. Design expert: real name, career at named companies, strong opinions, blind spots.
3. Write agent to `.claude/agents/<name>.md`.
4. Optionally register: `sdlc ponder team add <slug> --name ... --agent ...`.
5. **Next:** invoke with @<name> or continue session.
"#;

pub static SDLC_RECRUIT: CommandDef = CommandDef {
    slug: "sdlc-recruit",
    claude_content: SDLC_RECRUIT_COMMAND,
    gemini_description: "Recruit an expert thought partner as a persistent agent",
    playbook: SDLC_RECRUIT_PLAYBOOK,
    opencode_description: "Recruit an expert thought partner as a persistent agent",
    opencode_hint: "<role or domain context>",
    skill: SDLC_RECRUIT_SKILL,
};
