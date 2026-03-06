use crate::cmd::init::registry::CommandDef;

const SDLC_FIT_IMPACT_COMMAND: &str = r#"---
description: Evaluate a proposed architectural change against project vision, docs, and patterns — produces a structured go/no-go with blast radius map and preconditions
argument-hint: <proposal description>
allowed-tools: Bash, Read, Glob, Grep, Agent
---

# sdlc-fit-impact

Evaluate whether a proposed change fits the project's documented vision, follows established
patterns, and what it would touch if implemented. Produce a structured verdict with explicit
preconditions before any code gets written.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Principles

- **DOCS_ARE_THE_CONTRACT**: Vision, architecture, and project instructions are the measuring stick. If the proposal contradicts documented philosophy, that's a finding.
- **PRACTICE_OVER_THEORY**: What the codebase actually does matters as much as what docs say. Surface gaps between documented and actual patterns.
- **BLAST_RADIUS_IS_CONCRETE**: "This affects a lot" is not analysis. Every affected area gets a name, file paths, and a size estimate (S/M/L).
- **TENSIONS_ARE_FINDINGS**: When docs say one thing and the proposal implies another, surface it. The user decides which side adjusts.
- **PRECONDITIONS_BEFORE_CODE**: If docs need updating or decisions need making before implementation, say so.

---

## Steps

### 1. Frame the Proposal

Parse $ARGUMENTS into a concrete proposal statement:

```markdown
## Proposal

**Change:** [one-sentence description]
**Motivation:** [why this is being considered]
**Scope claim:** [what the proposer thinks it touches]
```

**Stop.** If the proposal cannot be stated in one sentence without hedging, ask the user to
sharpen it before proceeding. Do not analyze a vague proposal.

### 2. Load the Contract

Read the project's governing documents. Check for existence and read each:

```bash
cat VISION.md 2>/dev/null || echo "no VISION.md"
cat ARCHITECTURE.md 2>/dev/null || echo "no ARCHITECTURE.md"
cat CLAUDE.md 2>/dev/null || echo "no CLAUDE.md"
cat AGENTS.md 2>/dev/null || echo "no AGENTS.md"
ls docs/*.md 2>/dev/null
cat .sdlc/guidance.md 2>/dev/null || true
```

For each document, extract statements relevant to the proposal — both supporting and
contradicting. If a document doesn't exist, note it as a finding.

### 3. Audit Doc Alignment

For each governing document, evaluate alignment:

```markdown
## Doc Alignment

### [Document Name]
| Statement | Relationship | Finding |
|-----------|-------------|---------|
| "[exact quote]" | SUPPORTS / CONTRADICTS / SILENT | [implication for proposal] |

**Net assessment:** ALIGNED / TENSION / CONTRADICTION
```

Rules:
- Quote exact text, not paraphrases
- SILENT means docs don't address this area — that's a finding too
- If a document contradicts the proposal, state which should change and why
- If docs contradict each other, surface that as a pre-existing tension

### 4. Evaluate Pattern Fit

Find 2–3 existing patterns in the codebase that are most analogous to what the proposal
would introduce. Use Grep/Glob to find them, then read the relevant code.

```markdown
## Pattern Fit

### Existing Pattern: [Name]
**Location:** [file paths]
**How it works:** [brief]
**Proposal follows this pattern:** YES / PARTIALLY / NO
**Gap:** [what differs and why it matters]
```

Evaluate:
1. Does the proposal follow established patterns?
2. Does it introduce a new pattern? Is that justified?
3. Does it conflict with an existing pattern? Which should win?
4. Are there existing abstractions (traits, interfaces, modules) it should use?

### 5. Map the Blast Radius

Enumerate every area the change touches. Be concrete — use file paths.

```markdown
## Impact Map

| Area | Files/Modules | Change Type | Size | Notes |
|------|--------------|-------------|------|-------|
| [area] | [paths] | new / modify / delete / refactor | S / M / L | [specifics] |
```

Categories to check:
- **Core data model** — structs, types, traits, schemas
- **Storage layer** — databases, file I/O, migrations, backends
- **API surface** — endpoints, CLI commands, response shapes
- **Frontend** — components, types, API client, state management
- **Configuration** — env vars, config files, feature flags
- **Tests** — unit, integration, test helpers, fixtures
- **Documentation** — CLAUDE.md, ARCHITECTURE.md, README
- **Deployment** — CI/CD, Dockerfiles, Helm charts, migrations
- **Agent contracts** — slash commands, skill templates, AGENTS.md

Size guide:
- **S** — < 50 lines, isolated to one file/function
- **M** — 50–200 lines, touches 2–5 files, may need new tests
- **L** — 200+ lines, new module/crate, cross-cutting, needs migration

### 6. Surface Tensions & Risks

```markdown
## Tensions & Risks

### Tension: [Name]
**What docs say:** "[quote]"
**What practice does:** [observed behavior]
**What proposal implies:** [consequence]
**Resolution options:**
1. [option A] — update docs to reflect reality + proposal
2. [option B] — adjust proposal to match docs
3. [option C] — accept with documented rationale
```

Also flag:
- **Migration risks** — data transformations, backward compatibility
- **Rollback difficulty** — can this be undone? At what cost?
- **Ordering constraints** — what must happen before what?
- **Unknown unknowns** — areas where you lack visibility

### 7. Step Back

Before delivering the verdict, challenge the analysis:

1. **Am I reading docs too literally?** Does the philosophy actually prohibit this, or am I
   being too rigid? Was the statement written for this context?
2. **Did I miss blast radius?** Run additional searches for type names, function names, and
   config keys. Check indirect consumers.
3. **Is this the right unit of analysis?** Should I evaluate a larger or smaller scope?
4. **Am I being useful or just thorough?** Lead with verdict and key tensions. If there are
   20 findings but only 3 matter, say so.

Revise any section where step-back revealed a gap.

### 8. Deliver Preconditions & Verdict

```markdown
## Preconditions

Before implementation starts:

### Doc Updates Required
1. [Document] — [what needs to change and why]

### Decisions Required
1. [Decision] — [options, who decides, what blocks on this]

### Pattern Establishment
1. [Pattern] — [what needs to exist before new code follows it]

## Verdict

**Fit:** GOOD FIT / ACCEPTABLE WITH ADJUSTMENTS / POOR FIT
**Impact:** LOW / MODERATE / HIGH / CROSS-CUTTING
**Recommendation:** PROCEED / PROCEED AFTER PRECONDITIONS / RECONSIDER APPROACH
**Precondition count:** [N doc updates, M decisions, P patterns]
```

Verdict rules:
- PROCEED only if no unresolved doc contradictions and blast radius is understood
- PROCEED AFTER PRECONDITIONS if change fits but docs/decisions need updating first
- RECONSIDER APPROACH if proposal fundamentally contradicts project philosophy

---

### Next

| Context | Next |
|---|---|
| PROCEED | `**Next:** /sdlc-ponder-commit <slug>` — crystallize into milestones and features |
| PROCEED AFTER PRECONDITIONS | `**Next:** /sdlc-vision-adjustment` or `/sdlc-architecture-adjustment` — update docs first |
| RECONSIDER | `**Next:** /sdlc-ponder <slug>` — open ideation workspace to explore alternatives |
"#;

const SDLC_FIT_IMPACT_PLAYBOOK: &str = r#"# sdlc-fit-impact

Evaluate a proposed architectural change against project vision, docs, and patterns before implementation.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Frame the proposal — one sentence, motivation, scope claim. If vague, ask for clarity.
2. Read VISION.md, ARCHITECTURE.md, CLAUDE.md, AGENTS.md, docs/*.md, `.sdlc/guidance.md`.
3. Audit doc alignment — quote exact text, mark SUPPORTS / CONTRADICTS / SILENT per document.
4. Find 2–3 analogous existing patterns. Evaluate if proposal follows, extends, or conflicts.
5. Map blast radius — table of every area touched with file paths and S/M/L sizing.
6. Surface tensions — docs vs practice vs proposal, with resolution options.
7. Step back — challenge literal doc readings, check for missed blast radius, verify right scope.
8. Deliver verdict: PROCEED / PROCEED AFTER PRECONDITIONS / RECONSIDER. List all preconditions.
9. **Next:** `/sdlc-ponder-commit` (proceed), `/sdlc-vision-adjustment` (preconditions), or `/sdlc-ponder` (reconsider).
"#;

const SDLC_FIT_IMPACT_SKILL: &str = r#"---
name: sdlc-fit-impact
description: Evaluate a proposed architectural change against project vision, docs, and patterns. Produces doc alignment audit, pattern fit, blast radius map, tensions, and go/no-go verdict with preconditions.
---

# SDLC Fit & Impact Analysis

Evaluate whether a proposed change fits documented vision, follows patterns, and what it touches.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Frame the proposal as one sentence + motivation + scope claim. Reject vague proposals.
2. Read all governing docs: VISION.md, ARCHITECTURE.md, CLAUDE.md, AGENTS.md, docs/.
3. Audit doc alignment — quote exact text, mark SUPPORTS / CONTRADICTS / SILENT.
4. Find 2–3 analogous patterns. Evaluate if proposal follows, extends, or conflicts.
5. Map blast radius — every area with file paths and S/M/L sizing.
6. Surface tensions — docs vs practice vs proposal, with resolution options.
7. Step back — challenge readings, check missed blast radius, verify scope.
8. Deliver verdict: PROCEED / PROCEED AFTER PRECONDITIONS / RECONSIDER APPROACH.
9. **Next:** crystallize (ponder-commit), update docs first, or explore alternatives (ponder).
"#;

pub static SDLC_FIT_IMPACT: CommandDef = CommandDef {
    slug: "sdlc-fit-impact",
    claude_content: SDLC_FIT_IMPACT_COMMAND,
    gemini_description:
        "Evaluate a proposed change against vision, docs, and patterns — go/no-go with blast radius",
    playbook: SDLC_FIT_IMPACT_PLAYBOOK,
    opencode_description:
        "Evaluate a proposed change against vision, docs, and patterns — go/no-go with blast radius",
    opencode_hint: "<proposal description>",
    skill: SDLC_FIT_IMPACT_SKILL,
};
