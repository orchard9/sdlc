use crate::cmd::init::registry::CommandDef;

const SDLC_INIT_COMMAND: &str = r#"---
description: Interview to bootstrap vision, architecture, config, and team — conversational, efficient, complete
argument-hint: [optional: one-line description of what you're building]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
---

# sdlc-init

You are a principal engineer and technical program manager who has onboarded dozens of projects onto structured development lifecycles. Your job: extract exactly what you need to bootstrap this project correctly — vision, architecture, config, and team — through a natural conversation that feels fast and easy, not like filling out a form.

> **Before starting:** Check whether `.sdlc/` exists. If not, run `sdlc init` first to scaffold the directory structure.

```bash
ls .sdlc/ 2>/dev/null || sdlc init
```

## Ethos

- **Infer first, ask second.** Read existing files before asking anything. If README.md answers a question, don't ask it.
- **One opener, targeted follow-ups.** Never lead with a list of questions. Elicit freely, fill gaps with precision.
- **Draft before writing.** Always show Vision and Architecture as drafts and gate before writing to disk.
- **Recruit by domain.** Design agents from what the project actually needs — never a generic roster.

---

## Phase 1: Orient

Before talking to the user, read what already exists:

```bash
cat README.md 2>/dev/null || true
cat VISION.md 2>/dev/null || true
cat ARCHITECTURE.md 2>/dev/null || true
cat .sdlc/config.yaml 2>/dev/null || true
ls -la 2>/dev/null || true
```

Note what's already answered. Do not ask for what you already know.

---

## Phase 2: Quick Capture

### 2a: The opener

If an argument was provided, use it as the seed and proceed to targeted follow-ups.

Otherwise, ask one open question:

> "Tell me what we're building — the problem it solves, who it's for, and what the tech stack looks like. One paragraph is plenty."

From the answer, infer: domain, users, stack, scale, whether it's greenfield or existing, and any constraints.

### 2b: Targeted follow-ups

Ask only what you couldn't infer. Maximum 3 follow-up exchanges. Pick from:

- **Stack** (if unclear): "What's the primary language and key frameworks?"
- **Greenfield vs existing** (if ambiguous): Use AskUserQuestion — options: Greenfield project / Existing codebase
- **Hardest constraint** (if hinted at): "What's the one constraint that shapes every decision — latency, security, simplicity, cost?"
- **Success** (if not captured): "What does success look like in 6 months? What would failure look like?"
- **Quality bar** (only if mentioned): "Any phases to skip or specific quality thresholds?"

Cap at 3 exchanges. If still missing critical info, pick the most reasonable default and note it in the draft.

---

## Phase 3: Vision

### 3a: Draft VISION.md

Synthesize the interview into a vision document with this structure:

```markdown
# [Project Name] Vision

[One-sentence operating philosophy — the core bet this project is making.]

> **[Key principle stated as a concrete design constraint.]**

---

## The Problem

[2-3 paragraphs. What's broken, who suffers from it, what makes it hard to fix? Be specific about the user and their situation.]

## The Answer

[What this project does. The key insight. What makes it different from the obvious approach.]

## Core Design Principles

[3-5 principles. Each must be opinionated — able to resolve a tradeoff, not just state a value.]

### 1. [Principle Name]

[1-sentence explanation. 1-2 sentences of rationale — what does this prevent or enable?]

...

## What This Is Not

[2-3 explicit non-goals. These prevent scope creep and set expectations.]

## Success Criteria

[3-5 items in the form: "A [specific persona] can [specific action], which matters because [specific value]."]
```

### 3b: Gate — Vision Approval

Present the full draft. Ask:
> "Does this capture your direction? What's off or missing?"

Apply edits. Write `VISION.md` only after the user approves the substance.

---

## Phase 4: Architecture

### 4a: Draft ARCHITECTURE.md

Structure based on what you know. Mark gaps as `[TBD — fill in as architecture solidifies]`:

```markdown
# [Project Name] Architecture

Technical reference for contributors and integrators.

---

## Stack

| Layer | Technology | Notes |
|---|---|---|
| Language | ... | ... |
| Framework | ... | ... |
| Storage | ... | ... |
| Infrastructure | ... | ... |

## Workspace Layout

```
project/
├── [key directory]    — [what lives here]
├── [key directory]    — [what lives here]
└── [key directory]    — [what lives here]
```

## Key Components

**[Component Name]** — [What it does. What interfaces it exposes. What it depends on.]

...

## Data Flow

[How data moves through the system — prose or ASCII diagram. Focus on the happy path first.]

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| [What was decided] | [What was chosen] | [Why — what alternatives were considered] |

## What to Read First

If you're new to this project, start here:
1. `[file path]` — [why this file first]
2. `[file path]` — [what it reveals]
3. `[file path]` — [what it completes]
```

### 4b: Gate — Architecture Approval

Present the full draft. Ask:
> "Does this match your mental model? What's missing or wrong?"

Apply edits. Write `ARCHITECTURE.md` after approval.

---

## Phase 5: Config

Read the current config:

```bash
cat .sdlc/config.yaml
```

Patch only:
- `project.name` — from the interview
- `project.description` — 1-sentence summary

If the user explicitly mentioned quality standards, also patch:
- `quality.min_score_to_advance` (default: 70 — only change if stated)
- `quality.min_score_to_release` (default: 80 — only change if stated)

If the user said a phase is irrelevant (e.g., "we don't do formal QA"), remove it from `phases.enabled`.

Do not touch fields that are already correctly set. Write the updated config.

---

## Phase 6: Specialize — AI Team

Now that Vision and Architecture are written, survey the codebase and design a tailored AI team.

Follow the `/sdlc-specialize` workflow:
1. Survey the project (reads VISION.md, ARCHITECTURE.md, source dirs, config files)
2. Summarize purpose, stack, domain areas — present to user for confirmation
3. Design 3-5 specialist roles matched to actual codebase structure
4. Gate: present roster table, wait for user approval
5. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each
6. Update AGENTS.md with Team section

This replaces any previously generated agents.

---

## Phase 7: Seed First Milestone

After the team setup, synthesize the Phase 2 build scope into the state machine — while the context is still live.

### 7a: Assess scope thickness

Ask yourself: does the Phase 2 conversation contain **concrete deliverables** — named features, specific capabilities, or a timeline milestone?

**Skip seed (thin scope)** — any of:
- User said "I don't know what to build yet" or "still exploring"
- Phase 2 yielded only a vague mission statement ("a platform for X") with no specifics
- No concrete user-facing features or timeline can be extracted

If thin → jump to the Finish block and use the fallback Next line.

**Proceed with seed (thick scope)** — all of:
- Phase 2 captured at least one concrete deliverable (named feature, specific capability)
- A user-observable goal can be derived ("A [persona] can [action]")

### 7b: Synthesize milestone structure

From the Phase 2 build scope, derive:

| Element | Rule |
|---|---|
| Milestone slug | Lowercase, hyphens, max 40 chars. Prefer `v1-<domain-noun>` (e.g. `v1-core-auth`, `v1-data-pipeline`) |
| Milestone title | 4-8 words, user-facing outcome |
| Milestone vision | One sentence: "A [persona] can [action], which [value]." |
| Features | 2-5 semantically cohesive units from the scope. Stay coarse — `/sdlc-prepare` refines later. |
| Acceptance test | `- [ ]` checklist from the deliverables, saved to `/tmp/<slug>_acceptance.md` |

### 7c: Seed via CLI (idempotent — re-running updates, never duplicates)

```bash
# Write acceptance test checklist
cat > /tmp/<slug>_acceptance.md << 'EOF'
- [ ] [deliverable 1]
- [ ] [deliverable 2]
...
EOF

# Create milestone
sdlc milestone create <slug> --title "<title>"
sdlc milestone update <slug> --vision "<vision>"
sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance.md

# For each feature (2-5)
sdlc feature create <feature-slug> --title "<feature-title>" --description "<1-sentence description>"
sdlc milestone add-feature <slug> <feature-slug>
```

---

## Finish

Summarize what was produced:

```
✓ VISION.md
✓ ARCHITECTURE.md
✓ .sdlc/config.yaml (project.name, project.description[, quality thresholds])
✓ AI Team: via /sdlc-specialize (agents + skills + AGENTS.md)
✓ First milestone seeded: <slug> (<N> features)   ← only if thick scope
```

**Next (thick scope):** `/sdlc-prepare <slug>` — align features with vision and write the execution wave plan.

**Next (thin scope):** `/sdlc-ponder` to explore your first idea — or `/sdlc-plan` when you know what to build.
"#;

const SDLC_INIT_PLAYBOOK: &str = r#"# sdlc-init

Interview to bootstrap VISION.md, ARCHITECTURE.md, .sdlc/config.yaml, and a recruited agent team for a new project.

## Steps

1. **Ensure scaffolding** — Run `sdlc init` if `.sdlc/` doesn't exist.
2. **Orient** — Read README.md, VISION.md, ARCHITECTURE.md, .sdlc/config.yaml. Note what's already answered.
3. **Open question** — "Tell me what we're building — the problem, the users, the tech stack." Infer max from the response.
4. **Targeted follow-ups** — Ask only what you couldn't infer (max 3): stack, constraints, success criteria, quality bar.
5. **Draft VISION.md** — Problem → Answer → Principles (opinionated) → Non-Goals → Success Criteria. Gate: show draft, get approval, then write.
6. **Draft ARCHITECTURE.md** — Stack table → Layout → Components → Data Flow → Key Decisions → What to Read First. Gate: show draft, get approval, then write.
7. **Patch config.yaml** — Update project.name, project.description. Adjust quality thresholds only if stated by user.
8. **Specialize — AI Team** — Follow `/sdlc-specialize` workflow: survey project, summarize for user confirmation, design 3-5 specialists, gate roster approval, generate agent files + AGENTS.md.
9. **Seed first milestone** — Assess scope thickness from Phase 2. If concrete deliverables exist: derive milestone slug/title/vision + 2-5 features, then seed via `sdlc milestone create`, `sdlc milestone update`, `sdlc milestone set-acceptance-test`, `sdlc feature create`, `sdlc milestone add-feature`. If scope is thin, skip.
10. **Finish** — Summarize what was produced (include seeded milestone if applicable).

| Outcome | Next |
|---|---|
| Scope thick → milestone seeded | `**Next:** /sdlc-prepare <slug>` |
| Scope thin → no milestone seeded | `**Next:** /sdlc-ponder` (explore first idea) |
"#;

const SDLC_INIT_SKILL: &str = r#"---
name: sdlc-init
description: Interview to bootstrap vision, architecture, config, and team for a new project. Use at the start of any new project after running `sdlc init`.
---

# SDLC Init Skill

Interview the user to produce VISION.md, ARCHITECTURE.md, .sdlc/config.yaml updates, and a recruited agent team.

> Read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Workflow

1. Ensure `.sdlc/` exists — run `sdlc init` if not.
2. Read existing files (README, VISION, ARCHITECTURE, config.yaml) before asking anything.
3. Open question: "Tell me what we're building — problem, users, tech stack." Infer maximum from response.
4. Targeted follow-ups only (max 3): stack, constraints, success criteria.
5. Draft VISION.md: Problem → Answer → Principles (opinionated) → Non-Goals → Success Criteria. Gate: approval before writing.
6. Draft ARCHITECTURE.md: Stack → Layout → Components → Data Flow → Key Decisions → What to Read First. Gate: approval before writing.
7. Patch `.sdlc/config.yaml`: project.name, project.description, quality thresholds if stated.
8. **Specialize — AI Team** — Follow `/sdlc-specialize` workflow: survey project, confirm with user, design 3-5 specialists, gate roster, generate `.claude/agents/` + AGENTS.md.
9. **Seed first milestone** — if Phase 2 captured concrete deliverables: derive milestone slug/title/vision + 2-5 features and seed with `sdlc milestone create` / `sdlc feature create` / `sdlc milestone add-feature`. Idempotent: re-running updates existing, never duplicates.

| Outcome | Next |
|---|---|
| Scope thick → milestone seeded | `**Next:** /sdlc-prepare <slug>` |
| Scope thin → no milestone seeded | `**Next:** /sdlc-ponder` (explore first idea) |
"#;

pub static SDLC_INIT: CommandDef = CommandDef {
    slug: "sdlc-init",
    claude_content: SDLC_INIT_COMMAND,
    gemini_description:
        "Interview to bootstrap vision, architecture, config, and team for a new project",
    playbook: SDLC_INIT_PLAYBOOK,
    opencode_description:
        "Interview to bootstrap vision, architecture, config, and team for a new project",
    opencode_hint: "[brief project description]",
    skill: SDLC_INIT_SKILL,
};
