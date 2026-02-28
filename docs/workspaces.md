# Workspaces

Workspaces are the funnel into the work. Before a feature exists in the state machine, a workspace gives the idea, problem, or system a place to breathe — to be explored, interrogated, and synthesized into something concrete.

All workspace types share the same infrastructure: session-based dialogue, artifact capture, orientation frontmatter, and a phase model that drives agent behavior. The output of every workspace is actionable: a milestone, a feature, a guideline, or a decision.

---

## Types

### Ponder
*Ideation → milestones and features*

Free-form exploration space for new ideas. Recruits thought partners, builds a team around the idea, and captures the dialogue in sessions. When an idea is mature enough, committing it synthesizes the conversation into milestones and features in the state machine.

**Phases:** open-ended (no fixed sequence)
**Output:** milestones + features via `/sdlc-ponder-commit`
**Files:** `.sdlc/roadmap/<slug>/`
**Spec:** `ponder-space.md`, `ponder-dialogue.md`

---

### Root Cause
*Symptom → hypothesis → fix or guideline*

Structured forensic investigation. An agent interviews the problem through five parallel analysis areas (code paths, bottlenecks, data flow, auth chain, environment) then synthesizes a root cause hypothesis with a confidence score. Output is either a tracked fix task or a new guideline.

**Phases:** triage → investigate → synthesize → output
**Output:** fix feature (`fix-<slug>`) or guideline (`.sdlc/guidelines/<slug>.md`)
**Files:** `.sdlc/investigations/<slug>/`
**Spec:** `root-cause.md`

---

### Evolve
*System → evolution paths → roadmap → features*

Strategic improvement workspace for systems that work but need to grow. An agent surveys the system, scores it across five maturity lenses (pit of success, coupling, growth readiness, self-documenting, failure modes), identifies 2–4 evolution paths, and sequences them into a dependency-respecting roadmap. Each path becomes one or more features.

**Phases:** survey → analyze → paths → roadmap → output
**Output:** features per path, optionally a guideline
**Files:** `.sdlc/investigations/<slug>/` (kind: evolve)
**Spec:** `evolve.md`

---

### Guideline
*Recurring problem → principles → published document*

Pattern distillation workspace. Captures evidence of a recurring problem across the codebase (anti-patterns, good examples, prior art, adjacent patterns), extracts principles from the evidence, drafts a guideline document, and publishes it with optional enforcement (lint rule, PR template entry, CLAUDE.md reference).

**Phases:** problem → evidence → principles → draft → publish
**Output:** `.sdlc/guidelines/<slug>.md` + optional enforcement tasks
**Files:** `.sdlc/investigations/<slug>/` (kind: guideline)
**Spec:** `guidelines.md`

---

## Shared Infrastructure

All workspace types use the same underlying layer:

**Session protocol**
- Sessions are numbered markdown files with YAML frontmatter (`session`, `timestamp`, `orientation`)
- `orientation` has three fields: `current` (where we are), `next` (what happens next), `commit` (what unlocks the output)
- Agents write sessions to `/tmp/<type>-session-<slug>.md` then log via CLI — never write directly to the sessions directory
- CLI: `sdlc ponder session log <slug> --file /tmp/...` or `sdlc investigate session log <slug> --file /tmp/...`

**Artifact capture**
- Artifacts are plain markdown files stored alongside the manifest
- Captured via `sdlc ponder capture` / `sdlc investigate capture`
- Workspace panel in the UI renders artifacts with phase-aware context cards

**Phase model**
- Phase is a free string on the manifest — agents advance it by writing gate artifacts then calling `sdlc investigate update <slug> --phase <next>`
- Frontend renders a PhaseStrip showing progress through the sequence
- Each phase has an expected gate artifact; completing it unblocks advancement

**SSE live updates**
- Agent chat spawns a Claude run; SSE emits `*RunCompleted` when done
- Frontend reloads session list on completion — no polling

**Storage**
```
.sdlc/roadmap/<slug>/          ← ponder
  manifest.yaml
  team.yaml
  sessions/session-NNN.md
  <artifacts>.md

.sdlc/investigations/<slug>/   ← root-cause, evolve, guideline
  manifest.yaml                  (kind field discriminates)
  sessions/session-NNN.md
  <artifacts>.md
```

---

## Workspace → Work pipeline

```
Idea / Problem / System
        │
        ▼
   [workspace]
   ponder · root-cause · evolve · guideline
        │
        ▼
   Synthesize output
   milestones · features · guidelines · fix tasks
        │
        ▼
   State machine
   DRAFT → SPECIFIED → PLANNED → READY → IMPLEMENTATION → REVIEW → AUDIT → QA → MERGE
```

Workspaces are not optional pre-planning — they are the mechanism that ensures we build the right thing before the state machine locks us into building *something*.
