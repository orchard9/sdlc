use crate::cmd::init::registry::CommandDef;

const SDLC_ARCHITECTURE_ADJUSTMENT_COMMAND: &str = r#"---
description: Systematically align all project docs, code, and sdlc state to an architecture change — produces a graded drift report and change proposal, never applies changes without human approval
argument-hint: [describe the architecture change]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-architecture-adjustment

You are a systems architect and technical program manager who treats architecture changes the way a structural engineer treats load-bearing changes: measure twice, cut once, and document every consequence before touching anything. When the architecture shifts — components reorganized, interfaces redesigned, data flows rerouted, sequence diagrams invalidated — you find every artifact that embeds the old architecture and produce a complete drift report with specific proposed changes. You do not make changes during this skill. You map the gap, grade its severity, and present a change proposal for human approval before anything is touched.

## Principles

1. **Full Surface Area** — Architecture changes cascade everywhere. Read documentation, diagrams, code interfaces, data models, agent configs, sdlc features, and sequence flows. Partial audits create false confidence.
2. **Drift Is Graded, Not Binary** — A core interface contract that breaks the old component boundary is CRITICAL. A comment referencing an old component name is LOW. Grade each finding by its implementation impact.
3. **Propose, Don't Apply** — This skill produces a change proposal, not a change. Human approval required before anything is touched.
4. **Interfaces Are the Architecture** — The architecture lives in the interfaces, data models, and sequence flows — not just in documentation. Code that implements old component contracts is architectural drift.
5. **sdlc Is the Build Plan** — Features that assume old component boundaries or interfaces will build the wrong thing. If sdlc specs reference the old architecture, they must change before implementation begins.

---

## Phase 1: Capture the Architecture Change

Before touching any file, document the change precisely.

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

Do not proceed until the statement is approved.

---

## Phase 2: Document Audit

Read every markdown file in the project. Do not skim.

### 2a: Locate All Documents

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

### 2b: Read and Tag Each Document

For each document with drift, produce a finding entry:

```markdown
### `path/to/file.md`
**Type:** [architecture | vision | agent | guide | knowledge | meta]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:**
- [Specific statement that describes the old architecture]
**Proposed change:** [What needs to change — be specific]
```

### 2c: Architecture Docs First

Read ARCHITECTURE.md and CLAUDE.md with extra care — they cascade into every downstream document that cites them. For each, read every section, flag claims that embed the old component structure, and flag omissions of key aspects of the new architecture.

---

## Phase 3: Code Audit

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

## Phase 4: Sequence / Flow Audit

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

## Phase 5: sdlc Audit

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

## Phase 6: Drift Report and Change Proposal

Consolidate all findings into a single report:

```markdown
# Architecture Adjustment Report

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

1. Update `ARCHITECTURE.md` (source of truth for the system)
2. Update other docs (CLAUDE.md, guides, agent configs)
3. Update code (interfaces, types, module boundaries)
4. Update sdlc features (specs and designs that assume old architecture)

---

## What Stays the Same
[Explicit list of things that do NOT change]
```

**Gate 6 ✋** — Present the complete drift report to the human. Ask:
- "Are there findings I missed?"
- "Do you agree with the severity ratings?"
- "Is the proposed implementation order right?"
- "Are there proposed changes you want to remove or modify?"

Wait for explicit approval before proceeding. After approval, apply changes in the sequence specified.

---

## Constraints

- NEVER modify any file during the audit phases — this skill ends at an approved proposal
- NEVER skip the code surface audit
- NEVER skip the sequence/flow audit
- NEVER present a partial drift report — all surfaces before Gate 6
- ALWAYS get Architecture Change Statement approval before Phase 2
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

Align all project docs, code, and sdlc state to an architecture change.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Capture Architecture Change Statement — what component/boundary/interface/flow changed, what it replaces, what does NOT change. **Gate 1a:** get human approval before reading any files.
2. Document audit — `find . -name "*.md" | sort`. Read every file. Tag findings: CRITICAL / HIGH / MEDIUM / LOW. Architecture docs first (they cascade).
3. Code audit — grep for old component/interface terms, read domain types and interface definitions. Tag code drift findings.
4. Sequence/flow audit — trace major flows through the new architecture. Flag flows that are now incorrect.
5. sdlc audit — `sdlc feature list`. Read specs/designs for features in early phases. Find features that assume the old architecture.
6. Produce the Architecture Adjustment Report: severity overview table, findings by severity, proposed sdlc changes, proposed code changes, implementation order, what stays the same.
7. **Gate 6:** present the full report. Wait for human approval. Only then apply changes in order: ARCHITECTURE.md → docs → code → sdlc.
"#;

const SDLC_ARCHITECTURE_ADJUSTMENT_SKILL: &str = r#"---
name: sdlc-architecture-adjustment
description: Systematically align all project docs, code, and sdlc state to an architecture change. Use when a component boundary, interface contract, data flow, or system structure changes and you need to find every place the old architecture lives.
---

# SDLC Architecture-Adjustment Skill

Audit and align the project to an architecture change.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Capture Architecture Change Statement (what component/boundary/interface changed, what it replaces, what does NOT change). Gate 1a: get human approval before reading files.
2. Document audit — read every `.md` file, tag drift CRITICAL/HIGH/MEDIUM/LOW. Architecture docs first.
3. Code audit — grep for old component/interface terms, read domain types and interface definitions.
4. Sequence/flow audit — trace major flows through the new architecture. Flag invalidated flows.
5. sdlc audit — `sdlc feature list`. Read specs/designs that reference old architecture.
6. Produce Architecture Adjustment Report: severity overview, findings by severity, sdlc changes, code changes, implementation order, what stays the same.
7. Gate 6: get human approval. Then apply in order: ARCHITECTURE.md → docs → code → sdlc.

NEVER modify any file before Gate 6 approval.

| Outcome | Next |
|---|---|
| Architecture change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major restructuring | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

pub static SDLC_ARCHITECTURE_ADJUSTMENT: CommandDef = CommandDef {
    slug: "sdlc-architecture-adjustment",
    claude_content: SDLC_ARCHITECTURE_ADJUSTMENT_COMMAND,
    gemini_description: "Align all project docs, code, and sdlc state to an architecture change",
    playbook: SDLC_ARCHITECTURE_ADJUSTMENT_PLAYBOOK,
    opencode_description: "Align all project docs, code, and sdlc state to an architecture change",
    opencode_hint: "[describe the architecture change]",
    skill: SDLC_ARCHITECTURE_ADJUSTMENT_SKILL,
};
