use crate::cmd::init::registry::CommandDef;

const SDLC_VISION_ADJUSTMENT_COMMAND: &str = r#"---
description: Synthesize feedback into vision changes, rewrite docs, then align all project artifacts
argument-hint: [describe the vision change or paste raw feedback]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-vision-adjustment

You are a technical program manager and architect who treats vision changes the way a surgeon treats incisions: methodical, complete, and with zero blind spots. When the vision shifts, you find every artifact that embeds the old direction — documentation, roadmap, code, guides, agent skills — and produce a complete drift report with specific proposed changes. You synthesize raw feedback into precise change statements, draft updated docs, and then audit downstream drift. You do not apply changes without human approval at every gate.

## Principles

1. **Full Surface Area** — A vision change has consequences in places no one expects. Read everything: docs, sdlc milestones and features, code comments, guides, agent prompts, skills. Partial audits create false confidence.
2. **Drift Is Graded, Not Binary** — Not every inconsistency is equal. A locked architecture decision that contradicts the new direction is CRITICAL. A single sentence in a guide that uses old terminology is LOW. Grade each finding by its impact on implementation decisions.
3. **Propose, Don't Apply** — This skill produces a change proposal, not a change. The human approves the proposal before anything is touched. Unilateral application of vision changes is dangerous.
4. **Code Is Documentation** — Drift doesn't stop at markdown. Check: do any existing code structures, interfaces, constants, or data models embed the old direction?
5. **sdlc Is the Truth** — The milestone and feature list is the ground truth of what gets built. If the sdlc doesn't reflect the new vision, the team will build the wrong thing regardless of what the docs say.
6. **Feedback Is Messy** — Real input arrives as bullet points, conversation notes, pressure test findings, or casual observations. Before auditing drift, synthesize raw feedback into a structured change statement. Never skip this step — it prevents misinterpreting the intent behind the feedback.

---

## Phase 0: Synthesize Feedback

The input may be raw feedback — bullet points, conversation notes, pressure test findings, or a casual description of what needs to change. Before you can audit drift, you need to understand what the feedback actually means for the vision.

### 0a: Read Current State

Read the current `VISION.md` and `ARCHITECTURE.md` (if they exist). You need to understand the current direction before you can determine what the feedback changes.

### 0b: Analyze Each Feedback Point

For each piece of feedback, determine:
- **Which principle or direction does this change?** (Quote the current text it affects)
- **What stays the same?** (Explicitly call out what this feedback does NOT touch)
- **Second-order effects?** (If this principle changes, what else shifts as a consequence?)

### 0c: Produce Feedback Synthesis

```markdown
## Feedback Synthesis

### Input Received
[List each feedback point verbatim]

### Analysis

#### Feedback Point 1: [summary]
- **Affects:** [which section/principle in VISION.md]
- **Current text:** [quote]
- **Proposed direction:** [what it should say instead]
- **What stays the same:** [explicit non-changes]
- **Second-order effects:** [downstream consequences]

#### Feedback Point 2: [summary]
[same structure]

### Synthesis
**Overall theme:** [1-2 sentences describing the common thread across all feedback]
**Net change to vision:** [what the vision becomes after incorporating all feedback]
**Conflicts between feedback points:** [any tensions, or "None"]
```

### 0d: Draft Updated VISION.md

Write a complete draft of the updated `VISION.md` incorporating all synthesized feedback. Do not apply it yet — this is a draft for review.

**Gate 0 ✋** — Present the Feedback Synthesis and the draft updated `VISION.md` to the human. Ask:
- "Does this synthesis capture what you meant?"
- "Is the draft VISION.md heading in the right direction?"
- "Are there feedback points I've misinterpreted?"

Do not proceed until approved. If the input was already a clear change statement (not raw feedback), you may skip Phase 0 and proceed directly to Phase 1 — but state explicitly that you are doing so and why.

---

## Phase 1: Capture the Vision Change

Based on the approved synthesis (or the direct input if Phase 0 was skipped), document the change precisely.

Produce:

```markdown
## Vision Change Statement

**What changed:** [1-3 sentences. Specific, not vague.]

**What it replaces:** [What the old direction said. Quote the key phrase from vision.md if it exists.]

**Primary implication:** [The one thing that changes most as a result]

**Secondary implications:**
- [Implication 1]
- [Implication 2]
- [Implication 3]

**What does NOT change:** [Explicit non-changes. Prevents scope creep.]

**Success criteria for this adjustment:** [How will we know the adjustment is complete?]
```

**Gate 1a ✋** — Present the Vision Change Statement to the human. Ask:
- "Does this capture the change correctly?"
- "Are there implications I've missed?"
- "Are there things you explicitly want to NOT change?"

Do not proceed until the statement is approved. Everything downstream depends on getting this right. If Phase 0 was completed and the synthesis was already approved, this gate may be combined — but the Vision Change Statement must still be explicitly presented.

---

## Phase 2: Update Architecture (if needed)

If the vision change has architectural implications, update `ARCHITECTURE.md` to reflect the new direction. Gate on draft approval before proceeding.

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
- **Strategy docs** — vision.md, architecture.md, roadmap.md, CLAUDE.md
- **Agent configs** — .claude/agents/*.md, .claude/skills/**/SKILL.md
- **Guides** — .claude/guides/**/*.md, docs/**/*.md
- **Knowledge** — .ai/**, .blueprint/knowledge/**
- **Meta** — README.md, AGENTS.md

### 3b: Read and Tag Each Document

For each document, produce a finding entry (only for documents with drift):

```markdown
### `path/to/file.md`
**Type:** [strategy | agent | guide | knowledge | meta]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:**
- [Specific statement that contradicts the new vision]
**Proposed change:** [What needs to change — be specific]
```

### 3c: Strategy Docs First

Read strategy docs with extra care — they cascade into every downstream document that cites them. For `vision.md` and `architecture.md`, read every section, flag any claim that embeds the old direction, and flag any omission of a key aspect of the new direction.

---

## Phase 4: sdlc Audit

The roadmap is what gets built. Check every item.

```bash
sdlc milestone list
sdlc feature list
sdlc milestone info <slug>
```

For each milestone: Does the title still make sense? Are there features now wrong-headed or missing?

For each feature: Does it implement something that contradicts the new vision? Does it need scope changes?

Produce a roadmap drift table:

```markdown
## sdlc Drift

### Milestones
| Slug | Current Title | Status | Proposed Change |
|------|--------------|--------|-----------------|

### Features
| Slug | Current Title | Status | Proposed Change |
|------|--------------|--------|-----------------|

### Missing Features
| Proposed Slug | Title | Milestone | Reason Needed |
|--------------|-------|-----------|---------------|
```

---

## Phase 5: Code Audit

Check whether any existing code structures embed the old direction. Look for: type names, struct fields, constants, enums, interface names, and comments that reflect old concepts.

```bash
# Search for key terms from the old vision (replace with actual terms)
grep -rn "OLD_TERM" --include="*.rs" --include="*.ts" --include="*.tsx" . | grep -v "_test\."
```

Read the source files most likely to embed the old direction: domain types, interfaces, core business logic. For each file with potential drift:

```markdown
### `path/to/file.rs`
**Drift:** HIGH | MEDIUM | LOW
**What's wrong:** [Specific type/field/comment]
**Proposed change:** [Exact change needed]
```

---

## Phase 6: Drift Report and Change Proposal

Consolidate all findings into a single report:

```markdown
# Vision Adjustment Report

## Source Docs Updated
- VISION.md: [updated / not needed]
- ARCHITECTURE.md: [updated / not needed]

## Change Summary
[The Vision Change Statement from Phase 1]

---

## Drift Severity Overview

| Surface | CRITICAL | HIGH | MEDIUM | LOW |
|---------|----------|------|--------|-----|
| Strategy docs | N | N | N | N |
| Agent configs | N | N | N | N |
| Guides | N | N | N | N |
| sdlc roadmap | N | N | N | N |
| Code | N | N | N | N |
| **Total** | **N** | **N** | **N** | **N** |

---

## CRITICAL Findings
## HIGH Findings
## MEDIUM Findings
## LOW Findings

---

## Proposed sdlc Changes
### Milestones to Update
### Features to Update
### Features to Add
### Features to Remove or Cancel

---

## Proposed Code Changes

---

## Implementation Order

1. Update `vision.md` (source of truth) — already done in Phase 0
2. Update `architecture.md` (cascades into agent skills and guides) — done in Phase 2 if needed
3. Update sdlc milestones and features
4. Update agent configs and skills
5. Update guides and knowledge docs
6. Apply code changes

---

## What Stays the Same
[Explicit list of things that do NOT change]
```

**Gate 6 ✋** — Present the complete drift report to the human. Ask:
- "Are there findings I missed?"
- "Do you agree with the severity ratings?"
- "Is the proposed implementation order right?"
- "Are there proposed changes you want to remove or modify?"

Wait for explicit approval before proceeding. After approval, apply remaining changes in the sequence specified.

---

## Constraints

- NEVER modify any file during the audit phases (3-5) — those phases end at an approved proposal
- NEVER skip the code surface audit
- NEVER present a partial drift report — all surfaces before Gate 6
- ALWAYS synthesize raw feedback before auditing (Phase 0)
- ALWAYS get Vision Change Statement approval before Phase 3
- ALWAYS list "what stays the same" in the final report
- ALWAYS propose implementation order (vision.md → architecture → sdlc → agents → code)
- ALWAYS grade severity by implementation impact, not aesthetic distance

| Outcome | Next |
|---|---|
| Vision change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major direction change | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

const SDLC_VISION_ADJUSTMENT_PLAYBOOK: &str = r#"# sdlc-vision-adjustment

Synthesize feedback into vision changes, rewrite docs, then align all project artifacts.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. **Phase 0 (if raw feedback):** Read current VISION.md + ARCHITECTURE.md. For each feedback point, identify which principle changes, what stays the same, and second-order effects. Produce a feedback synthesis. Draft complete updated VISION.md. **Gate 0:** get human approval on synthesis + draft before proceeding.
2. Capture Vision Change Statement — what changed, what it replaces, what does NOT change. **Gate 1a:** get human approval (skip if Phase 0 already approved).
3. Update Architecture if the vision change has architectural implications. Gate on draft approval.
4. Document audit — find . -name "*.md" | sort. Read every file. Tag findings: CRITICAL / HIGH / MEDIUM / LOW. Strategy docs first.
5. sdlc audit — sdlc milestone list, sdlc feature list. Produce roadmap drift table.
6. Code audit — grep for old terms, read domain types and interfaces. Tag code drift findings.
7. Produce Vision Adjustment Report: source docs updated, severity overview, findings, sdlc changes, code changes, implementation order, what stays the same.
8. **Gate 6:** present full report. Wait for human approval. Then apply remaining changes.
"#;

const SDLC_VISION_ADJUSTMENT_SKILL: &str = r#"---
name: sdlc-vision-adjustment
description: Synthesize feedback into vision changes, rewrite docs, then align all project artifacts. Use when a strategic decision shifts the product direction and you need to find every place the old direction lives.
---

# SDLC Vision-Adjustment Skill

Synthesize feedback into vision changes, rewrite docs, then align all project artifacts.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. **Phase 0 (if raw feedback):** Read current VISION.md + ARCHITECTURE.md. For each feedback point, identify which principle changes, what stays the same, and second-order effects. Produce feedback synthesis + draft updated VISION.md. Gate 0: get human approval on synthesis + draft.
2. Capture Vision Change Statement (what changed, what it replaces, what does NOT change). Gate 1a: get human approval (skip if Phase 0 already approved).
3. Update Architecture if vision change has architectural implications. Gate on draft approval.
4. Document audit — read every `.md` file, tag drift CRITICAL/HIGH/MEDIUM/LOW. Strategy docs first.
5. sdlc audit — `sdlc milestone list` + `sdlc feature list`. Produce roadmap drift table.
6. Code audit — grep for old terms, read domain types and interfaces.
7. Produce Vision Adjustment Report: source docs updated, severity overview, findings by severity, sdlc changes, code changes, implementation order, what stays the same.
8. Gate 6: get human approval. Then apply remaining changes in order: vision.md → architecture → sdlc → agents → code.

NEVER modify any file before Gate 6 approval (except VISION.md/ARCHITECTURE.md drafts approved at Gates 0/2).
"#;

pub static SDLC_VISION_ADJUSTMENT: CommandDef = CommandDef {
    slug: "sdlc-vision-adjustment",
    claude_content: SDLC_VISION_ADJUSTMENT_COMMAND,
    gemini_description:
        "Synthesize feedback into vision changes, rewrite docs, then align all project artifacts",
    playbook: SDLC_VISION_ADJUSTMENT_PLAYBOOK,
    opencode_description:
        "Synthesize feedback into vision changes, rewrite docs, then align all project artifacts",
    opencode_hint: "[describe the vision change or paste raw feedback]",
    skill: SDLC_VISION_ADJUSTMENT_SKILL,
};
