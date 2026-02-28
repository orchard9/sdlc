# Evolve Interface

## What it is

A system evolution workspace. The user points at a system, area of code, or architectural pattern; the interface runs a structured analysis across five maturity lenses and produces either concrete implementation tasks (features, urgently staged) or a guideline documenting the target architecture.

This is the `systems-evolutionist` + `fix-forward` skill pair expressed as an interactive workspace. The key insight: evolution produces a recommendation (what should change and why), but the work gets executed through the normal task/feature pipeline, not ad-hoc.

---

## Phase Model

```
Survey → Analyze → Paths → Roadmap → Output
```

| Phase | What happens | Agent does |
|-------|-------------|-----------|
| **Survey** | Understand the system structure, entry points, pain points, what's documented | Reads files, writes `survey.md` — structure, entry points, docs state, TODOs/FIXMEs |
| **Analyze** | Apply five maturity lenses; score each | Writes `lens-analysis.md` with maturity table and key gaps per lens |
| **Paths** | Propose 2–4 evolution paths with effort/impact | Writes `paths.md` — each path with before/after, concrete changes, effort/impact bars |
| **Roadmap** | Sequence paths by correctness dependency (not effort) | Writes `roadmap.md` — proper solution → enabling changes → extended vision |
| **Output** | Decide: tasks or guideline | Creates features or writes architecture doc |

---

## Perspectives (Maturity Lenses)

During Analyze phase, five lenses are applied. Each produces a maturity rating:

| # | Lens | Question | Maturity levels |
|---|------|---------|----------------|
| 1 | Pit of Success | Do defaults lead to good outcomes? | Low / Medium / High / Excellent |
| 2 | Coupling | Are related things together? | Low / Medium / High / Excellent |
| 3 | Growth Readiness | Will this scale to 10x? | Low / Medium / High / Excellent |
| 4 | Self-Documenting | Can you understand it from the system itself? | Low / Medium / High / Excellent |
| 5 | Failure Modes | What happens when it breaks? | Low / Medium / High / Excellent |

The lens cards in the workspace panel show:
- Lens name
- Current maturity (color-coded: red=Low, amber=Medium, green=High, emerald=Excellent)
- One-line key gap
- Populated progressively as the agent analyzes

During Paths phase, lens cards pivot to path cards:
- Path name and vision sentence
- Effort bar (1–5 blocks, filled)
- Impact bar (1–5 blocks, filled)
- Which lens(es) it addresses
- Blocked by / enables (dependency arrow)

---

## UI Structure

```
┌──────────────────────────────────────────────────────────┐
│ ← Title                              [status] [📁]       │
├──────────────────────────────────────────────────────────┤
│ [Survey] → [Analyze] → [Paths] → [Roadmap] → [Output]   │  ← PhaseStrip
├─────────────────────────────────────┬────────────────────┤
│                                     │                    │
│  Session dialogue stream            │  Lens cards        │  ← Analyze phase
│                                     │  (switch to path   │
│                                     │   cards in Paths,  │
│                                     │   roadmap tree     │
│                                     │   in Roadmap)      │
│                                     │                    │
├─────────────────────────────────────┴────────────────────┤
│  Describe what to evolve / answer questions...           │
└──────────────────────────────────────────────────────────┘
```

**Workspace panel adapts per phase:**

| Phase | Workspace panel content |
|-------|------------------------|
| Survey | File/artifact browser (standard) |
| Analyze | Five lens cards with maturity + gap |
| Paths | Path cards with effort/impact visualization |
| Roadmap | Three-tier checklist (proper / enabling / extended) |
| Output | OutputGate (create tasks or write guideline) |

This is the key design extension vs Ponder: the workspace panel is phase-aware and shows structured data (not just files) during analysis phases. Files remain accessible via a tab or secondary view.

---

## Data Model

```yaml
# .sdlc/investigations/<slug>/manifest.yaml
slug: api-layer-hexagonal
title: "API Layer — Hexagonal Architecture Evolution"
kind: evolve               # discriminator field (root_cause | evolve | guideline)
phase: analyze             # survey | analyze | paths | roadmap | output | done
status: in_progress
scope: "crates/sdlc-server/src/"   # what system is being analyzed
created_at: "..."
updated_at: "..."
lens_scores:               # populated during analyze phase
  pit_of_success: medium
  coupling: low
  growth_readiness: medium
  self_documenting: low
  failure_modes: medium
output_refs: []            # list of feature slugs or guideline paths
```

All investigations share a flat directory — `kind` in the manifest discriminates the type.

Artifacts:
```
.sdlc/investigations/<slug>/
  manifest.yaml
  sessions/
    session-001.md
  survey.md
  lens-analysis.md
  paths.md
  roadmap.md
```

---

## Output Types

### Implementation Tasks
For each path selected for execution, creates an urgently-staged feature:

```bash
sdlc feature create hexagonal-ports --title "Establish port/adapter pattern in API layer"
# The roadmap becomes the spec
sdlc artifact draft hexagonal-ports spec
```

Features are linked back to the evolve investigation. Multiple paths → multiple features, staged by dependency order from the roadmap.

### Architecture Guideline
Writes `.sdlc/guidelines/<slug>.md`:

```markdown
# <Architecture Pattern Title>

**System scope:** Where this applies.
**Problem:** What goes wrong without this.

## Target Architecture

[Vision statement from the evolution path]

## Rules
1. [Concrete rule from principles]
2. ...

## Patterns

### Do
```code example```

### Don't
```code example```

## Evolution Path
[Roadmap checklist extracted from paths.md]

## Evidence
- Evolved from: [investigation slug]
```

---

## Relationship Between Root Cause and Evolve

These two interfaces are often sequential:

```
Root Cause → (root cause is systemic design) → Evolve → implementation tasks
Root Cause → (root cause is isolated bug) → Fix task directly
```

An investigation can promote to an evolution session. When the root cause synthesis reveals a design problem (not just a bug), the OutputGate shows: "This is a systemic issue. Evolve the system instead?" — linking the root cause investigation as the context for a new evolve session.

---

## Agent Prompt

```
You are running a systems evolution analysis.
Current phase: {phase}
Scope: {scope}
Title: {title}

[embed systems-evolutionist SKILL.md content]
[embed fix-forward SKILL.md for output phase depth recommendations]
```

Step-back questions from the skill are enforced: the agent challenges each evolution path for YAGNI, hidden stakeholders, and execution reality before moving to Roadmap phase.

---

## CLI Commands

```bash
sdlc investigate create <slug> --kind evolve --title "..." --context "..."
sdlc investigate update <slug> --scope "crates/sdlc-server/src/"
sdlc investigate capture <slug> --content "..." --as lens-analysis.md
sdlc investigate session log <slug> --file /tmp/investigation-session-<slug>.md
```

Session protocol is identical to root-cause (see root-cause.md).

---

## Implementation Order

1. **Data layer** — `InvestigationEntry` struct with `kind: evolve`, `scope`, `lens_scores` fields — **DONE** (`sdlc-core/src/investigation.rs`)
2. **`LensCards` component** — five maturity cards in workspace panel
3. **`PathCards` component** — effort/impact visualization
4. **`RoadmapChecklist` component** — three-tier checklist in workspace
5. **Phase-aware workspace panel** — `WorkspacePanel` receives `phase` prop, renders appropriate view
6. **List + detail page** — `/investigations/evolve`
7. **OutputGate extension** — multi-task output (one feature per selected path)
8. **Agent prompt** — wire systems-evolutionist skill
