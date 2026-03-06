use crate::cmd::init::registry::CommandDef;

const SDLC_ARCHITECTURE_ADJUSTMENT_COMMAND: &str = r#"---
description: Synthesize feedback into architecture changes, rewrite docs, then align all project artifacts
argument-hint: [describe the architecture change or paste raw feedback]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-architecture-adjustment

You are a systems architect and technical program manager who treats architecture changes the way a structural engineer treats load-bearing changes: measure twice, cut once, and document every consequence before touching anything. When the architecture shifts — components reorganized, interfaces redesigned, data flows rerouted, sequence diagrams invalidated — you find every artifact that embeds the old architecture and produce a complete drift report with specific proposed changes. You synthesize raw feedback into precise change statements, draft updated docs, and then audit downstream drift. You do not apply changes without human approval at every gate.

## Principles

1. **Full Surface Area** — Architecture changes cascade everywhere. Read documentation, diagrams, code interfaces, data models, agent configs, sdlc features, and sequence flows. Partial audits create false confidence.
2. **Drift Is Graded, Not Binary** — A core interface contract that breaks the old component boundary is CRITICAL. A comment referencing an old component name is LOW. Grade each finding by its implementation impact.
3. **Propose, Don't Apply** — This skill produces a change proposal, not a change. Human approval required before anything is touched.
4. **Interfaces Are the Architecture** — The architecture lives in the interfaces, data models, and sequence flows — not just in documentation. Code that implements old component contracts is architectural drift.
5. **sdlc Is the Build Plan** — Features that assume old component boundaries or interfaces will build the wrong thing. If sdlc specs reference the old architecture, they must change before implementation begins.
6. **Feedback Is Messy** — Real input arrives as bullet points, conversation notes, pressure test findings, or casual observations. Before auditing drift, synthesize raw feedback into a structured change statement. Never skip this step — it prevents misinterpreting the intent behind the feedback.

---

## Phase 0: Synthesize Feedback

The input may be raw feedback — bullet points, conversation notes, pressure test findings, or a casual description of what needs to change. Before you can audit drift, you need to understand what the feedback actually means for the architecture.

### 0a: Read Current State

Read the current `ARCHITECTURE.md` and `VISION.md` (if they exist). You need to understand the current system structure before you can determine what the feedback changes.

### 0b: Analyze Each Feedback Point

For each piece of feedback, determine:
- **Which component, boundary, or interface does this change?** (Quote the current text it affects)
- **What stays the same?** (Explicitly call out what this feedback does NOT touch)
- **Second-order effects?** (If this component/interface changes, what else shifts as a consequence?)

### 0c: Produce Feedback Synthesis

```markdown
## Feedback Synthesis

### Input Received
[List each feedback point verbatim]

### Analysis

#### Feedback Point 1: [summary]
- **Affects:** [which section/component in ARCHITECTURE.md]
- **Current text:** [quote]
- **Proposed direction:** [what it should say instead]
- **What stays the same:** [explicit non-changes]
- **Second-order effects:** [downstream consequences — interfaces, data flows, sequences]

#### Feedback Point 2: [summary]
[same structure]

### Synthesis
**Overall theme:** [1-2 sentences describing the common thread across all feedback]
**Net change to architecture:** [what the architecture becomes after incorporating all feedback]
**Conflicts between feedback points:** [any tensions, or "None"]
```

### 0d: Draft Updated ARCHITECTURE.md

Write a complete draft of the updated `ARCHITECTURE.md` incorporating all synthesized feedback. Do not apply it yet — this is a draft for review.

**Gate 0 ✋** — Present the Feedback Synthesis and the draft updated `ARCHITECTURE.md` to the human. Ask:
- "Does this synthesis capture what you meant?"
- "Is the draft ARCHITECTURE.md heading in the right direction?"
- "Are there feedback points I've misinterpreted?"

Do not proceed until approved. If the input was already a clear change statement (not raw feedback), you may skip Phase 0 and proceed directly to Phase 1 — but state explicitly that you are doing so and why.

---

## Phase 1: Capture the Architecture Change

Based on the approved synthesis (or the direct input if Phase 0 was skipped), document the change precisely.

Produce:

```markdown
## Architecture Change Statement

**What changed:** [1-3 sentences. Specific: which component, boundary, interface, or flow changed.]

**What it replaces:** [What the old architecture said. Quote the key description if ARCHITECTURE.md exists.]

**Primary implication:** [The one thing that changes most as a result]

**Secondary implications:**
- [Implication 1 — component boundary change]
- [Implication 2 — interface contract change]
- [Implication 3 — data flow change]
- [Implication 4 — sequence diagram change]

**What does NOT change:** [Explicit non-changes. Prevents scope creep.]

**Success criteria for this adjustment:** [How will we know the adjustment is complete?]
```

**Gate 1a ✋** — Present the Architecture Change Statement to the human. Ask:
- "Does this capture the change correctly?"
- "Are there components or interfaces I've missed?"
- "Are there things you explicitly want to NOT change?"

Do not proceed until the statement is approved. If Phase 0 was completed and the synthesis was already approved, this gate may be combined — but the Architecture Change Statement must still be explicitly presented.

---

## Phase 2: Update Vision (if needed)

If the architecture change has vision implications, update `VISION.md` to reflect the new direction. Gate on draft approval before proceeding.

---

## Phase 3: Document Audit

Read every markdown file in the project. Do not skim.

### 3a: Locate All Documents

```bash
find . -name "*.md" \
  -not -path "*/node_modules/*" \
  -not -path "*/.git/*" \
  -not -path "*/vendor/*" \
  | sort
```

Categorize by type:
- **Architecture docs** — ARCHITECTURE.md, CLAUDE.md, any diagram files
- **Vision docs** — VISION.md, roadmap.md
- **Agent configs** — .claude/agents/*.md, .claude/skills/**/SKILL.md
- **Guides** — .claude/guides/**/*.md, docs/**/*.md
- **Knowledge** — .ai/**, .blueprint/knowledge/**
- **Meta** — README.md, AGENTS.md

### 3b: Read and Tag Each Document

For each document with drift, produce a finding entry:

```markdown
### `path/to/file.md`
**Type:** [architecture | vision | agent | guide | knowledge | meta]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:**
- [Specific statement that describes the old architecture]
**Proposed change:** [What needs to change — be specific]
```

### 3c: Architecture Docs First

Read ARCHITECTURE.md and CLAUDE.md with extra care — they cascade into every downstream document that cites them. For each, read every section, flag claims that embed the old component structure, and flag omissions of key aspects of the new architecture.

---

## Phase 4: Code Audit

Find code that implements old component boundaries, interface contracts, data models, or dependency patterns.

```bash
# Search for key terms from the old architecture (replace with actual terms)
grep -rn "OLD_COMPONENT" --include="*.rs" --include="*.ts" --include="*.tsx" . | grep -v "_test\."
```

Look specifically for:
- Type names, struct fields, enums, and constants that reflect old component names
- Interface definitions that embed old contracts
- Import paths that reference old module boundaries
- Comments that describe old data flows

For each file with potential drift:

```markdown
### `path/to/file.rs`
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:** [Specific type/field/interface/comment]
**Proposed change:** [Exact change needed]
```

---

## Phase 5: Sequence / Flow Audit

Identify flows that are now incorrect under the new architecture.

For each major user-facing flow or agent workflow, trace the path:
- Which components are involved?
- Which interfaces are called?
- Which data models are passed?

Flag any flow where the old sequence is no longer valid:

```markdown
### Flow: [Name]
**Old sequence:** [brief description]
**New sequence:** [brief description under the new architecture]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What breaks:** [What will fail if not updated]
```

---

## Phase 6: sdlc Audit

Features that assume the old architecture in their spec, design, or tasks will build the wrong thing.

```bash
sdlc milestone list
sdlc feature list
```

For each feature in draft/specified/planned phases: Does its spec or design describe old component boundaries, old interfaces, or old data flows?

Produce an sdlc drift table:

```markdown
## sdlc Drift

### Features with Architectural Assumptions
| Slug | Current Phase | Artifact | What's Wrong | Proposed Change |
|------|--------------|----------|--------------|-----------------|

### Missing Features (new work created by the architecture change)
| Proposed Slug | Title | Milestone | Reason Needed |
|--------------|-------|-----------|---------------|
```

---

## Phase 7: Drift Report and Change Proposal

Consolidate all findings into a single report:

```markdown
# Architecture Adjustment Report

## Source Docs Updated
- ARCHITECTURE.md: [updated / not needed]
- VISION.md: [updated / not needed]

## Change Summary
[The Architecture Change Statement from Phase 1]

---

## Drift Severity Overview

| Surface | CRITICAL | HIGH | MEDIUM | LOW |
|---------|----------|------|--------|-----|
| Architecture docs | N | N | N | N |
| Code interfaces | N | N | N | N |
| Sequence flows | N | N | N | N |
| Agent configs | N | N | N | N |
| sdlc roadmap | N | N | N | N |
| **Total** | **N** | **N** | **N** | **N** |

---

## CRITICAL Findings
## HIGH Findings
## MEDIUM Findings
## LOW Findings

---

## Proposed sdlc Changes
### Features to Update
### Features to Add
### Features to Remove or Cancel

---

## Proposed Code Changes

---

## Implementation Order

1. Update `ARCHITECTURE.md` (source of truth for the system) — already done in Phase 0
2. Update `VISION.md` if needed — done in Phase 2 if needed
3. Update other docs (CLAUDE.md, guides, agent configs)
4. Update code (interfaces, types, module boundaries)
5. Update sdlc features (specs and designs that assume old architecture)

---

## What Stays the Same
[Explicit list of things that do NOT change]
```

**Gate 7 ✋** — Present the complete drift report to the human. Ask:
- "Are there findings I missed?"
- "Do you agree with the severity ratings?"
- "Is the proposed implementation order right?"
- "Are there proposed changes you want to remove or modify?"

Wait for explicit approval before proceeding. After approval, apply remaining changes in the sequence specified.

---

## Constraints

- NEVER modify any file during the audit phases (3-6) — those phases end at an approved proposal
- NEVER skip the code surface audit
- NEVER skip the sequence/flow audit
- NEVER present a partial drift report — all surfaces before Gate 7
- ALWAYS synthesize raw feedback before auditing (Phase 0)
- ALWAYS get Architecture Change Statement approval before Phase 3
- ALWAYS list "what stays the same" in the final report
- ALWAYS propose implementation order (ARCHITECTURE.md → docs → code → sdlc)
- ALWAYS grade severity by implementation impact, not aesthetic distance

| Outcome | Next |
|---|---|
| Architecture change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major restructuring | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

const SDLC_ARCHITECTURE_ADJUSTMENT_PLAYBOOK: &str = r#"# sdlc-architecture-adjustment

Synthesize feedback into architecture changes, rewrite docs, then align all project artifacts.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. **Phase 0 (if raw feedback):** Read current ARCHITECTURE.md + VISION.md. For each feedback point, identify which component/boundary/interface changes, what stays the same, and second-order effects. Produce a feedback synthesis. Draft complete updated ARCHITECTURE.md. **Gate 0:** get human approval on synthesis + draft before proceeding.
2. Capture Architecture Change Statement — what component/boundary/interface/flow changed, what it replaces, what does NOT change. **Gate 1a:** get human approval (skip if Phase 0 already approved).
3. Update Vision if the architecture change has vision implications. Gate on draft approval.
4. Document audit — find . -name "*.md" | sort. Read every file. Tag findings: CRITICAL / HIGH / MEDIUM / LOW. Architecture docs first.
5. Code audit — grep for old component/interface terms, read domain types and interface definitions. Tag code drift findings.
6. Sequence/flow audit — trace major flows through the new architecture. Flag flows that are now incorrect.
7. sdlc audit — sdlc feature list. Read specs/designs for features in early phases. Find features that assume old architecture.
8. Produce Architecture Adjustment Report: source docs updated, severity overview, findings, sdlc changes, code changes, implementation order, what stays the same.
9. **Gate 7:** present full report. Wait for human approval. Then apply remaining changes.
"#;

const SDLC_ARCHITECTURE_ADJUSTMENT_SKILL: &str = r#"---
name: sdlc-architecture-adjustment
description: Synthesize feedback into architecture changes, rewrite docs, then align all project artifacts. Use when a component boundary, interface contract, data flow, or system structure changes and you need to find every place the old architecture lives.
---

# SDLC Architecture-Adjustment Skill

Synthesize feedback into architecture changes, rewrite docs, then align all project artifacts.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. **Phase 0 (if raw feedback):** Read current ARCHITECTURE.md + VISION.md. For each feedback point, identify which component/boundary/interface changes, what stays the same, and second-order effects. Produce feedback synthesis + draft updated ARCHITECTURE.md. Gate 0: get human approval on synthesis + draft.
2. Capture Architecture Change Statement (what component/boundary/interface changed, what it replaces, what does NOT change). Gate 1a: get human approval (skip if Phase 0 already approved).
3. Update Vision if architecture change has vision implications. Gate on draft approval.
4. Document audit — read every `.md` file, tag drift CRITICAL/HIGH/MEDIUM/LOW. Architecture docs first.
5. Code audit — grep for old component/interface terms, read domain types and interface definitions.
6. Sequence/flow audit — trace major flows through the new architecture. Flag invalidated flows.
7. sdlc audit — `sdlc feature list`. Read specs/designs that reference old architecture.
8. Produce Architecture Adjustment Report: source docs updated, severity overview, findings by severity, sdlc changes, code changes, implementation order, what stays the same.
9. Gate 7: get human approval. Then apply remaining changes in order: ARCHITECTURE.md → docs → code → sdlc.

NEVER modify any file before Gate 7 approval (except ARCHITECTURE.md/VISION.md drafts approved at Gates 0/2).

| Outcome | Next |
|---|---|
| Architecture change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major restructuring | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

pub static SDLC_ARCHITECTURE_ADJUSTMENT: CommandDef = CommandDef {
    slug: "sdlc-architecture-adjustment",
    claude_content: SDLC_ARCHITECTURE_ADJUSTMENT_COMMAND,
    gemini_description: "Synthesize feedback into architecture changes, rewrite docs, then align all project artifacts",
    playbook: SDLC_ARCHITECTURE_ADJUSTMENT_PLAYBOOK,
    opencode_description: "Synthesize feedback into architecture changes, rewrite docs, then align all project artifacts",
    opencode_hint: "[describe the architecture change or paste raw feedback]",
    skill: SDLC_ARCHITECTURE_ADJUSTMENT_SKILL,
};
