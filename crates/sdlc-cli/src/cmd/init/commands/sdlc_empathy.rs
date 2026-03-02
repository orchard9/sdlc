use crate::cmd::init::registry::CommandDef;

const SDLC_EMPATHY_COMMAND: &str = r#"---
description: Interview user perspectives deeply — surface needs, friction, deal-breakers, and conflicts before making decisions
argument-hint: <feature, system, or decision to evaluate>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task, AskUserQuestion
---

# sdlc-empathy

Run deep empathy interviews against a feature, system, or decision. Identifies specific
user personas, interviews each with probing questions, synthesizes findings, and surfaces
conflicts that reveal design tensions.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Ethos

- **Users over builders.** What we want to build matters less than what users need.
- **Absence is information.** If we can't find a perspective, that's a gap to acknowledge.
- **Conflicts are gold.** Disagreement between personas reveals tensions to resolve.
- **Empathy requires effort.** Quick assumptions aren't empathy. Deep interviews are.

---

## Steps

### 1. Identify stakeholders

For the subject in question, identify 3-5 specific personas:
1. **Primary user** — hands on keyboard daily
2. **Indirect stakeholder** — affected downstream (ops, support, consumers)
3. **Adoption blocker** — skeptic or reluctant user
4. **Forgotten voice** — new user, edge case, accessibility need

Be specific: not "developer" but "developer debugging a production issue at 2am."

### 2. Find or create perspective agents

For each persona, check if an agent exists. If missing, recruit one using the
recruitment protocol — write a perspective agent to `.claude/agents/<persona>-perspective.md`.

**PAUSE if a critical perspective is missing.** Surface the gap to the user before
proceeding blind.

### 3. Deep interview each perspective

For each persona, ask across five dimensions:

**Context:** "Walk me through your typical day when you'd interact with this."
**Needs:** "What problem are you solving? What does success look like?"
**Friction:** "What would make you sigh? Give up? Try something else?"
**Delight:** "What would make you think 'they get it'?"
**Deal-breakers:** "What would make you refuse to use this? Actively complain?"

### 4. Synthesize

| Analysis | What to surface |
|---|---|
| Alignments | Needs shared across 3+ personas |
| Conflicts | Where personas disagree — these are the most valuable |
| Gaps | Needs we didn't anticipate |
| Overbuilding | Things we planned that no persona actually wants |

### 5. Step back

- **Bias check** — did we hear uncomfortable truths, or only what we wanted?
- **Quiet voice** — whose perspective was easiest to ignore?
- **Stress test** — what if each persona is right and we're wrong?
- **Humility** — what don't we know that we don't know?

### 6. Recommend

Evidence-based recommendations tied to specific interview findings.
Acknowledge tradeoffs — who loses and why.
Flag what still needs real user validation.

---

### 7. Capture (if in a ponder session)

```bash
sdlc ponder capture <slug> --file /tmp/perspectives.md --as perspectives.md
```

### 8. Next

| Context | Next |
|---|---|
| Within a ponder session | `**Next:** /sdlc-ponder <slug>` |
| Pre-pressure-test | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Standalone for a feature | `**Next:** /sdlc-run <feature-slug>` |
"#;

const SDLC_EMPATHY_PLAYBOOK: &str = r#"# sdlc-empathy

Run deep empathy interviews to surface user needs and conflicts.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Identify 3-5 specific personas: primary user, indirect stakeholder, adoption blocker, forgotten voice.
2. For each, create a perspective agent if missing.
3. Deep interview across: context, needs, friction, delight, deal-breakers.
4. Synthesize: alignments, conflicts, gaps, overbuilding.
5. Step back: bias check, quiet voice, stress test, humility.
6. Recommend with evidence. Acknowledge tradeoffs.
7. If in ponder session: `sdlc ponder capture <slug> --file /tmp/perspectives.md --as perspectives.md`.
8. **Next:** continue ponder, pressure-test, or run.
"#;

const SDLC_EMPATHY_SKILL: &str = r#"---
name: sdlc-empathy
description: Interview user perspectives deeply to surface needs, friction, and conflicts. Use before making design decisions or when pressure-testing scope.
---

# SDLC Empathy Skill

Use this skill to run deep empathy interviews.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Identify 3-5 specific personas for the subject.
2. Create perspective agents if missing.
3. Deep interview: context, needs, friction, delight, deal-breakers.
4. Synthesize: alignments, conflicts, gaps, overbuilding.
5. Step back: bias, quiet voice, stress test, humility.
6. Recommend with evidence. If in ponder, capture as perspectives.md.
"#;

pub static SDLC_EMPATHY: CommandDef = CommandDef {
    slug: "sdlc-empathy",
    claude_content: SDLC_EMPATHY_COMMAND,
    gemini_description: "Interview user perspectives deeply before making decisions",
    playbook: SDLC_EMPATHY_PLAYBOOK,
    opencode_description: "Interview user perspectives deeply before making decisions",
    opencode_hint: "<feature, system, or decision>",
    skill: SDLC_EMPATHY_SKILL,
};
