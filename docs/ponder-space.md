# Ponder Space

Pre-milestone ideation workspace for exploring and committing ideas into the state machine.

---

## Problem

The sdlc state machine starts at `Draft`. Before Draft, there is nothing. But real work happens before Draft: someone walks in with a half-formed idea — "I need to build a database that dynamically stores personal preferences with cohort layering, constantly updating..." — and that idea needs to be interrogated, explored, prototyped, and shaped before it becomes a milestone with features.

Today, this work happens in ephemeral conversation. Insights evaporate between sessions. Thought partners are spawned fresh each time. There is no canonical place to accumulate ideation artifacts, no way to track which ideas are being explored, and no structured path from "I have an idea" to "milestones and features exist in the state machine."

The ideation skills exist (`/thinkthrough`, `/empathy`, `/propose`, `/prototype`, `/recruit`, `/evolve`) but they don't compose — each starts fresh, each produces output that lives and dies in conversation context.

## Design Principles

**Workspace, not workflow.** `/sdlc-ponder` opens a sketchbook. It doesn't start a pipeline. Ideation is inherently non-deterministic — sometimes you prototype first, sometimes you start with empathy. The system tracks what exists and suggests what's missing, but never enforces sequence.

**Ephemeral conversation, persistent artifacts.** The thinking happens in conversation. The artifacts — briefs, problem framings, research findings, prototypes, decisions — persist in the scrapbook. Next session, the conversation is new but the accumulated thinking isn't.

**Thought partners, not one-shot consultants.** Recruited experts are bound to a specific idea. They persist across sessions. When you come back to an idea, the team is still there and the scrapbook gives them continuity.

**Storage inside sdlc, intelligence in skills.** The `.sdlc/roadmap/` directory, manifest format, and CLI commands live in sdlc. The facilitation, interrogation, synthesis, and recruitment live in slash command templates. Same separation as features — sdlc tracks state, consumers do the thinking.

**Everything in git.** Scrapbook artifacts are committed. The history of how an idea evolved is part of the project record.

---

## Concepts

### Ponder Entry

A named workspace for exploring a single idea. Lives at `.sdlc/roadmap/<slug>/`. Has a manifest, a team, and a scrapbook of artifacts.

### Scrapbook

The collection of artifacts inside a ponder entry. No enforced structure. Files accumulate as thinking progresses. The scrapbook is a directory, not a data model — you can toss things in, rename them, reorganize them.

### Thought Partners

Agents recruited via `/sdlc-recruit` and bound to a specific ponder entry via `team.yaml`. They provide persistent expertise and pushback across sessions. When the ponder command loads a workspace, it loads the team.

### Committing

The act of crystallizing a pondered idea into structured input for `/sdlc-plan`. Reads the entire scrapbook, synthesizes it with the recruited team, and produces milestones, features, and tasks that enter the state machine. "Commit" as in committing to the idea — deciding "yes, this is what I want to build."

---

## Storage Model

### Directory structure

```
.sdlc/roadmap/<slug>/
  manifest.yaml              # entry metadata
  team.yaml                  # recruited thought partners
  brief.md                   # raw idea as stated (verbatim capture)
  <any-name>.md              # scrapbook artifacts — no enforced naming
```

### manifest.yaml

```yaml
slug: preference-engine
title: Dynamic preference system with cohort layering
status: exploring           # exploring | converging | committed | parked
created_at: 2026-02-27T10:00:00Z
updated_at: 2026-02-27T14:30:00Z
committed_at: null            # set when /sdlc-ponder-commit runs
committed_to:                 # milestone slugs created by commit
  - null
tags:                        # freeform, for search/filtering
  - personalization
  - data-layer
sessions: 3                  # count of ponder sessions (informational)
```

**Status semantics:**
- `exploring` — actively being pondered. Default on creation.
- `converging` — direction chosen, narrowing toward a plan. Set manually or by `/sdlc-ponder-commit` precondition check.
- `committed` — successfully fed into `/sdlc-plan`. Milestones/features exist in the state machine. The scrapbook remains as historical record.
- `parked` — idea explored and set aside. Not deleted — might be revisited. Like archiving a feature.

### team.yaml

```yaml
partners:
  - name: kai-tanaka
    role: Preference systems architect
    context: "Built Spotify's preference engine. Strong opinions about layered config vs flat key-value."
    agent: .claude/agents/kai-tanaka.md
    recruited_at: 2026-02-27T10:15:00Z
  - name: maya-okonkwo
    role: End-user advocate
    context: "Represents the developer consuming this preference API day-to-day."
    agent: .claude/agents/maya-okonkwo.md
    recruited_at: 2026-02-27T10:15:00Z
  - name: alex-dubois
    role: Pragmatic skeptic
    context: "Challenges whether this needs to be built at all. Asks about existing solutions."
    agent: .claude/agents/alex-dubois.md
    recruited_at: 2026-02-27T10:20:00Z
```

### Scrapbook artifacts

No enforced naming or types. Common patterns that will emerge:

| Filename | Typical content | Produced by |
|---|---|---|
| `brief.md` | Verbatim raw idea as first stated | `/sdlc-ponder` (day 1 capture) |
| `problem.md` | Reframed problem after interrogation | Costorm with thought partners |
| `landscape.md` | Existing solutions, adjacent systems, prior art | Research within ponder session |
| `exploration.md` | Solution options, trade-offs, architecture sketches | Costorm convergence |
| `prototype-notes.md` | Description/link to prototype, learnings | After prototyping within ponder session |
| `perspectives.md` | User research findings | After `/sdlc-empathy` or empathy protocol within ponder |
| `decisions.md` | Key decisions made, options considered, rationale | Accumulated across sessions |
| `plan.md` | Structured plan ready for `/sdlc-plan` | `/sdlc-ponder-commit` output |

Users can create any file with any name. The system reads whatever is there.

---

## CLI Commands

New `sdlc ponder` namespace.

### `sdlc ponder create <slug> --title "<title>"`

Create a new ponder entry. Creates `.sdlc/roadmap/<slug>/manifest.yaml` with status `exploring`. Optionally `--brief "<text>"` to write `brief.md` inline.

### `sdlc ponder list [--status <status>]`

List all ponder entries with slug, title, status, artifact count, team size, last updated. Default: all non-parked entries. `--status parked` to see parked ideas.

JSON output includes the full manifest for each entry.

### `sdlc ponder show <slug>`

Show a ponder entry: manifest fields, team members, and a listing of all scrapbook artifacts (filename, size, last modified). Reads and displays `brief.md` content if present.

JSON output includes manifest, team, and artifact file listing.

### `sdlc ponder capture <slug> --file <path> [--as <filename>]`

Copy a file into the scrapbook. If `--as` is provided, rename on copy. If the file is `-`, read from stdin. This is how artifacts from external tools (prototypes, research docs) enter the scrapbook.

### `sdlc ponder capture <slug> --content "<text>" --as <filename>`

Write inline content directly to a scrapbook artifact. For quick captures during conversation.

### `sdlc ponder team add <slug> --name "<name>" --role "<role>" --context "<context>" --agent <path>`

Register a recruited thought partner. Appends to `team.yaml`. The `--agent` path is relative to project root (e.g., `.claude/agents/kai-tanaka.md`).

### `sdlc ponder team list <slug>`

List the recruited team for a ponder entry.

### `sdlc ponder update <slug> [--status <status>] [--title "<title>"]`

Update ponder entry metadata. Status transitions: `exploring` <-> `converging`, any -> `parked`, `converging` -> `committed` (only via `/sdlc-ponder-commit`, not directly).

### `sdlc ponder archive <slug>`

Shorthand for `sdlc ponder update <slug> --status parked`.

### `sdlc ponder artifacts <slug>`

List all files in the scrapbook with size, modification time. Useful for programmatic access.

---

## Slash Commands

Four new commands installed by `sdlc init` / `sdlc update` to all four platforms (Claude Code, Gemini CLI, OpenCode, Agents).

### `/sdlc-ponder [slug]`

The workspace entry point. Sets the context for a creative ideation session. Embeds ideation, empathy, and recruitment capabilities natively — these do not depend on external skills.

```markdown
---
description: Open the ideation workspace — explore ideas with recruited thought partners, capture artifacts in the scrapbook, commit when ready. Embeds ideation, empathy, and recruitment protocols natively.
argument-hint: [slug or new idea description]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task, AskUserQuestion
---

# sdlc-ponder

Open the ponder workspace for creative exploration. This command sets the context —
from here, everything is conversation. You have access to all thinking tools. Artifacts
you produce land in the scrapbook and persist across sessions.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Entering the workspace

### If $ARGUMENTS is a known slug

```bash
sdlc ponder show <slug>
```

Read the manifest, team, and all scrapbook artifacts. Load the team's agent definitions.
Summarize where the idea stands: what's been captured, who's on the team, what hasn't
been explored yet.

### If $ARGUMENTS looks like a new idea (not an existing slug)

1. Derive a slug from the idea text (lowercase, hyphens, max 40 chars).
2. Create the entry:
```bash
sdlc ponder create <slug> --title "<derived title>"
sdlc ponder capture <slug> --content "<verbatim user text>" --as brief.md
```
3. Read the brief. Identify domain signals. Recruit 2-3 initial thought partners using
   the recruit skill — always include:
   - A domain expert (someone who's built something like this before)
   - An end-user advocate (who uses what this produces?)
   - A pragmatic skeptic (should this exist at all?)
4. Register them:
```bash
sdlc ponder team add <slug> --name "<name>" --role "<role>" \
  --context "<why this person>" --agent <agent-path>
```

### If no arguments

```bash
sdlc ponder list
```

Show all active ponder entries. Ask the user what they want to explore.

---

## During the session

You are a facilitator running a collaborative thinking session. The recruited team
members are your co-thinkers — channel their expertise and perspectives.

### What you do naturally

- **Interrogate the brief.** Push past the stated solution to find the real problem.
  "You said database — what problem does the database solve? Who reads these preferences?
  At what scale? What happens when cohort preferences conflict with individual ones?"
- **Channel thought partners.** Don't just think as yourself — voice the perspectives
  of recruited team members. "Kai would push back here — layered config inheritance is
  notoriously hard to debug. Have you thought about how a developer traces why a
  preference has a particular value?"
- **Suggest captures.** When a breakthrough happens — a reframing, a key decision, a
  constraint surfaced — offer to capture it: "That reframing is important. Should I
  capture it as problem.md in the scrapbook?"
- **Surface what's missing.** Track which dimensions of the idea have been explored.
  Problem framing? User perspectives? Technical landscape? Solution options? Decisions?
  Gently surface gaps: "We've been deep on the data model but haven't talked about who
  the users of this system actually are."

### Capturing artifacts

When something is worth persisting:

```bash
# Write inline content
sdlc ponder capture <slug> --content "<markdown content>" --as <filename>.md

# Or write to a temp file first for larger artifacts
# (write the file, then capture it)
sdlc ponder capture <slug> --file /tmp/exploration.md --as exploration.md
```

### Recruiting additional partners

If a new domain surfaces ("oh, this also needs a real-time sync layer"), recruit:

```bash
# Use the recruit skill to create the agent, then register them
sdlc ponder team add <slug> --name "<name>" --role "<role>" \
  --context "<context>" --agent .claude/agents/<name>.md
```

### Embedded capabilities

The ponder command carries these protocols natively. No external skills required.

#### Ideation protocol (embedded from ideate pattern)

When exploring a problem:
1. **Understand** — capture the problem statement, your interpretation, scope, success criteria
2. **Gather context** — read relevant code, specs, adjacent systems
3. **Synthesize** — landscape, constraints, gaps, key files
4. **Consult thought partners** — channel each recruited expert's perspective
5. **Explore solutions** — 3-4 options including "do nothing", with trade-offs
6. **Step back** — assumption audit, fresh eyes test, skeptic's questions, reversal
7. **Think out loud** — share learnings, surprises, core tension, questions
8. **Collaborate** — listen, adjust, iterate with the user

#### Empathy protocol (embedded from empathy-interviewer pattern)

When exploring user perspectives:
1. **Identify stakeholders** — direct users, indirect, blockers, forgotten
2. **Create perspective agents** — specific humans in specific situations, not abstract "users"
3. **Deep interview each** — context, needs, friction, delight, deal-breakers
4. **Synthesize** — alignments, conflicts, gaps, surprises
5. **Step back** — bias check, quiet voice, stress test, humility check
6. **Recommend** — evidence-based, tradeoffs acknowledged, unknowns flagged

Always include at least 3 perspectives. Always include a skeptic.

#### Recruitment protocol (embedded from recruit pattern)

When a domain signal emerges and you need a thought partner:
1. **Orient** — what expertise is needed and why
2. **Design the expert** — real name, career background at named companies, specific
   technical philosophy, strong opinions (not generic advice)
3. **Create the agent** — write to `.claude/agents/<name>.md` with background, principles,
   ALWAYS/NEVER rules, and "When Stuck" section
4. **Register** — `sdlc ponder team add <slug> --name ... --agent ...`

#### Feature shaping protocol (embedded from feature-proposer pattern)

When an idea starts converging toward something buildable:
1. **Seed** — working name, one-liner, hypothesis, trigger
2. **User perspectives** — who uses this, who's affected, who's skeptical
3. **Expert consultation** — technical feasibility, architecture fit, constraints
4. **Shape** — core value prop, user stories, design decisions, trade-offs
5. **Define MVP** — minimum lovable, not minimum viable
6. **Step back** — do we need this? scope creep? quiet voices heard?

Use this during convergence sessions and as the core of `/sdlc-ponder-commit`.

---

## Ending the session

Summarize what was explored, what was captured, and what remains unexplored.

Always end with **Next:**

| State | Next |
|---|---|
| Early exploration, many gaps | `**Next:** /sdlc-ponder <slug>` (continue exploring) |
| Direction emerging, need depth | `**Next:** /sdlc-ponder <slug>` (continue with focus on <gap>) |
| Idea shaped, ready to commit | `**Next:** /sdlc-ponder-commit <slug>` |
| Idea explored and parked | `**Next:** /sdlc-ponder` (explore something else) |
```

### `/sdlc-ponder-commit <slug>`

Crystallize a pondered idea into the state machine.

```markdown
---
description: Commit to a pondered idea — crystallize it into milestones and features via sdlc-plan
argument-hint: <ponder-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-ponder-commit

Commit to a pondered idea. Reads the entire scrapbook, synthesizes with the recruited
thought partners using the feature-shaping protocol, and produces milestones, features,
and tasks that enter the state machine. The bridge from sketchbook to blueprint.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Prerequisites

A ponder entry should have enough substance to commit. Not a rigid checklist, but the
agent should assess:

- Is the problem understood? (not just the solution)
- Have user perspectives been considered?
- Are the key technical decisions made?
- Is the scope defined?

If substance is thin, say so and suggest `/sdlc-ponder <slug>` to continue exploring.

---

## Steps

### 1. Load the scrapbook

```bash
sdlc ponder show <slug> --json
```

Read every artifact in the scrapbook. Read the team definitions. Build full context.

### 2. Load existing sdlc state

```bash
sdlc milestone list --json
sdlc feature list --json
```

Understand what already exists — avoid duplicating milestones or features.

### 3. Synthesize

With the full scrapbook and team context, determine the right structure:

**Small idea** (single capability, fits in one feature) →
- One feature, possibly added to an existing milestone
- Tasks decomposed from the exploration/decisions artifacts

**Medium idea** (multiple related capabilities) →
- One milestone with 2-5 features
- Vision synthesized from the problem framing and user perspectives

**Large idea** (significant initiative, multiple delivery phases) →
- Multiple milestones with clear ordering
- Each milestone has a user-observable goal

Present the proposed structure to the user:
> "Based on the scrapbook, here's what I see:
> - Milestone: <title> — <vision>
>   - Feature: <title> — <description>
>   - Feature: <title> — <description>
> Does this capture what you want to build?"

### 4. Produce the plan

Write a structured plan to the scrapbook:

```bash
sdlc ponder capture <slug> --file /tmp/<slug>-plan.md --as plan.md
```

The plan format matches what `/sdlc-plan` expects: milestones with features and tasks.

### 5. Distribute via sdlc-plan

Feed the plan into the state machine:

```bash
# sdlc-plan reads the plan and creates milestones/features/tasks
```

Execute the `/sdlc-plan` flow with `--file .sdlc/roadmap/<slug>/plan.md`.

### 6. Update the ponder entry

```bash
sdlc ponder update <slug> --status committed
```

Record which milestones were created (update `committed_to` in manifest).

### 7. Report

Show what was created: milestones, features, tasks. Link back to the scrapbook for
historical context.

---

### 8. Next

| Outcome | Next |
|---|---|
| Single feature created | `**Next:** /sdlc-run <feature-slug>` |
| Milestone created | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Multiple milestones | `**Next:** /sdlc-pressure-test <first-milestone-slug>` |
| Plan needs refinement | `**Next:** /sdlc-ponder <slug>` (back to exploring) |
```

### `/sdlc-recruit <role or context>`

Standalone recruitment command. Creates expert agents for thought partnership, implementation, or user perspective. Usable within ponder sessions, pressure tests, or independently.

```markdown
---
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
  technical contributions. Not "experienced engineer" but "built Stripe's webhook
  retry system, then led config inheritance at Spotify."
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
```

### `/sdlc-empathy <subject>`

Standalone empathy interview command. Deep user perspective interviews usable within ponder sessions, before pressure tests, or independently.

```markdown
---
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
```

---

## Server Routes

New route module: `roadmap.rs`.

### `GET /api/roadmap`

List all ponder entries. Returns array of manifests with artifact counts and team sizes. Mirrors `GET /api/milestones` pattern.

Response shape:
```json
[
  {
    "slug": "preference-engine",
    "title": "Dynamic preference system with cohort layering",
    "status": "exploring",
    "created_at": "2026-02-27T10:00:00Z",
    "updated_at": "2026-02-27T14:30:00Z",
    "artifact_count": 4,
    "team_size": 3,
    "tags": ["personalization", "data-layer"],
    "sessions": 3
  }
]
```

### `GET /api/roadmap/:slug`

Full ponder entry detail. Returns manifest, team, and artifact listing with content.

Response shape:
```json
{
  "slug": "preference-engine",
  "title": "...",
  "status": "exploring",
  "tags": [...],
  "sessions": 3,
  "created_at": "...",
  "updated_at": "...",
  "committed_at": null,
  "committed_to": [],
  "team": [
    {
      "name": "kai-tanaka",
      "role": "Preference systems architect",
      "context": "...",
      "agent": ".claude/agents/kai-tanaka.md"
    }
  ],
  "artifacts": [
    {
      "filename": "brief.md",
      "size_bytes": 342,
      "modified_at": "2026-02-27T10:05:00Z",
      "content": "I need to build a database that..."
    },
    {
      "filename": "problem.md",
      "size_bytes": 1204,
      "modified_at": "2026-02-27T11:30:00Z",
      "content": "## Problem Framing\n\nThe core problem is not..."
    }
  ]
}
```

### `POST /api/roadmap`

Create a ponder entry.

Request: `{ "slug": "preference-engine", "title": "...", "brief": "optional initial text" }`

### `POST /api/roadmap/:slug/capture`

Capture an artifact. Used by the frontend for direct text entry.

Request: `{ "filename": "decisions.md", "content": "## Key Decisions\n\n..." }`

### `PUT /api/roadmap/:slug`

Update manifest fields (title, status, tags).

---

## Frontend

### New page: Roadmap (`/roadmap`)

Grid of ponder entry cards, similar to MilestonesPage. Each card shows:
- Title and slug
- Status badge (exploring / converging / committed / parked)
- Tag pills
- Artifact count and team size as subtle metadata
- Last updated timestamp
- If committed: link to the milestone(s) it produced

Filter tabs: Exploring | Converging | Committed | Parked

### New page: Ponder Detail (`/roadmap/:slug`)

Two-column layout on desktop, stacked on mobile.

**Left column: Scrapbook**
- List of artifacts as expandable cards (collapsed by default, show filename + modified date)
- Click to expand and show rendered markdown content (reuse `MarkdownContent` component)
- Fullscreen button per artifact (reuse `FullscreenModal`)
- If `brief.md` exists, always show it expanded at the top

**Right column: Team & Meta**
- Status badge with update control
- Team members listed with name, role, and one-line context
- Tags (displayed as pills)
- Session count
- Created / updated timestamps
- If committed: "Committed into" section with links to milestone(s)
- Action buttons:
  - Copy `/sdlc-ponder <slug>` command (clipboard button)
  - Copy `/sdlc-ponder-commit <slug>` command (clipboard button)

### Sidebar update

Add "Roadmap" nav item between "Milestones" and "Archive" in Sidebar.tsx:

```
Dashboard
Features
Milestones
Roadmap       ← new
Archive
Config
```

Icon: `Lightbulb` from lucide-react (or `Compass` — the metaphor of exploration).

### Dashboard integration

Add a "Pondering" section to the Dashboard, between the stats bar and the milestone sections. Shows active ponder entries (status `exploring` or `converging`) as compact cards with:
- Title
- Artifact count (e.g., "4 artifacts")
- Team size (e.g., "3 partners")
- Copy button for `/sdlc-ponder <slug>`

Only renders if there are active ponder entries. This gives visibility to ideation work alongside feature execution work.

### Search integration

Add ponder entries to the search index. The `SearchModal` should find ponder entries by title, tags, and artifact content. Results link to `/roadmap/:slug`.

This requires a new `PonderIndex` in the search system (mirroring `FeatureIndex`) or extending the existing query endpoint.

### TypeScript types

Add to `frontend/src/lib/types.ts`:

```typescript
export type PonderStatus = 'exploring' | 'converging' | 'committed' | 'parked';

export interface PonderSummary {
  slug: string;
  title: string;
  status: PonderStatus;
  tags: string[];
  artifact_count: number;
  team_size: number;
  sessions: number;
  created_at: string;
  updated_at: string;
  committed_at: string | null;
  committed_to: string[];
}

export interface PonderTeamMember {
  name: string;
  role: string;
  context: string;
  agent: string;
  recruited_at: string;
}

export interface PonderArtifact {
  filename: string;
  size_bytes: number;
  modified_at: string;
  content: string;
}

export interface PonderDetail {
  slug: string;
  title: string;
  status: PonderStatus;
  tags: string[];
  sessions: number;
  created_at: string;
  updated_at: string;
  committed_at: string | null;
  committed_to: string[];
  team: PonderTeamMember[];
  artifacts: PonderArtifact[];
}
```

### API client additions

Add to `frontend/src/api/client.ts`:

```typescript
// Roadmap / Ponder
getRoadmap: () => request<PonderSummary[]>('/api/roadmap'),
getPonderEntry: (slug: string) => request<PonderDetail>(`/api/roadmap/${slug}`),
createPonderEntry: (data: { slug: string; title: string; brief?: string }) =>
  request<PonderDetail>('/api/roadmap', { method: 'POST', body: JSON.stringify(data) }),
updatePonderEntry: (slug: string, data: Partial<{ title: string; status: PonderStatus; tags: string[] }>) =>
  request<PonderDetail>(`/api/roadmap/${slug}`, { method: 'PUT', body: JSON.stringify(data) }),
capturePonderArtifact: (slug: string, data: { filename: string; content: string }) =>
  request<void>(`/api/roadmap/${slug}/capture`, { method: 'POST', body: JSON.stringify(data) }),
```

### SSE integration

The existing SSE mechanism (polling `.sdlc/state.yaml` mtime) needs to also detect changes in `.sdlc/roadmap/`. Two options:

**Option A (simple):** Also poll mtime of `.sdlc/roadmap/` directory. Any file change in any ponder entry triggers an SSE "update" event. Frontend hooks that care about roadmap data re-fetch.

**Option B (targeted):** Add a separate SSE event type "roadmap_update" so feature hooks don't re-fetch unnecessarily.

Recommend Option A for simplicity — the 800ms poll is already lightweight, and re-fetching is cheap.

---

## Core Library Changes

### New module: `crates/sdlc-core/src/ponder.rs`

Types and CRUD operations for ponder entries.

```rust
// Types
pub struct PonderEntry {
    pub slug: String,
    pub title: String,
    pub status: PonderStatus,
    pub tags: Vec<String>,
    pub sessions: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub committed_at: Option<DateTime<Utc>>,
    pub committed_to: Vec<String>,
}

pub enum PonderStatus {
    Exploring,
    Converging,
    Committed,
    Parked,
}

pub struct PonderTeamMember {
    pub name: String,
    pub role: String,
    pub context: String,
    pub agent: String,
    pub recruited_at: DateTime<Utc>,
}

pub struct PonderArtifactMeta {
    pub filename: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
}

// Operations
pub fn create_ponder(root: &Path, slug: &str, title: &str) -> Result<PonderEntry>;
pub fn load_ponder(root: &Path, slug: &str) -> Result<PonderEntry>;
pub fn list_ponders(root: &Path) -> Result<Vec<PonderEntry>>;
pub fn update_ponder(root: &Path, slug: &str, updates: PonderUpdate) -> Result<PonderEntry>;
pub fn capture_artifact(root: &Path, slug: &str, filename: &str, content: &[u8]) -> Result<()>;
pub fn list_artifacts(root: &Path, slug: &str) -> Result<Vec<PonderArtifactMeta>>;
pub fn read_artifact(root: &Path, slug: &str, filename: &str) -> Result<String>;
pub fn load_team(root: &Path, slug: &str) -> Result<Vec<PonderTeamMember>>;
pub fn add_team_member(root: &Path, slug: &str, member: PonderTeamMember) -> Result<()>;
```

### Path additions

Add to `crates/sdlc-core/src/paths.rs`:

```rust
pub fn roadmap_dir(root: &Path) -> PathBuf {
    root.join(ROADMAP_DIR)
}

pub fn ponder_dir(root: &Path, slug: &str) -> PathBuf {
    roadmap_dir(root).join(slug)
}

pub fn ponder_manifest(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join(MANIFEST_FILE)
}

pub fn ponder_team(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join("team.yaml")
}
```

### Search additions

Add `PonderIndex` to `crates/sdlc-core/src/search.rs` for full-text search over ponder entries. Index fields: `slug`, `title`, `tags`, `status`, `body` (concatenation of all scrapbook artifact content). Mirrors the `FeatureIndex` pattern.

### State integration

The project-level `.sdlc/state.yaml` should be aware of active ponder entries. Add an `active_ponders` field to the `State` struct:

```yaml
active_ponders:
  - slug: preference-engine
    title: Dynamic preference system with cohort layering
    status: exploring
```

This keeps the state file as the single source for "what's happening in this project" — features, milestones, AND active ideation.

---

## CLI Module

### New module: `crates/sdlc-cli/src/cmd/ponder.rs`

Subcommands: `create`, `list`, `show`, `capture`, `team` (with `add`/`list`), `update`, `archive`, `artifacts`.

Pattern: follows the `milestone.rs` command structure. Uses `print_json()` for `--json` output, `print_table()` for human-readable output.

---

## Init / Update Changes

### New command templates

Add to `crates/sdlc-cli/src/cmd/init.rs`:

- `SDLC_PONDER_COMMAND` / `PLAYBOOK` / `SKILL` — workspace entry point with embedded ideation, empathy, recruitment protocols
- `SDLC_PONDER_COMMIT_COMMAND` / `PLAYBOOK` / `SKILL` — crystallize idea into milestones/features with embedded feature-shaping protocol
- `SDLC_RECRUIT_COMMAND` / `PLAYBOOK` / `SKILL` — standalone expert recruitment
- `SDLC_EMPATHY_COMMAND` / `PLAYBOOK` / `SKILL` — standalone deep empathy interviews

That's 4 commands x 3 variants = 12 new const strings.

Register all in the four `write_user_*` functions. Add filenames to `migrate_legacy_project_scaffolding()`.

The ponder and ponder-commit templates will be the largest (~200+ lines for Claude Code variant) because they embed full protocols. This is consistent with `/sdlc-pressure-test` and `/sdlc-specialize` which are already substantial.

### AGENTS.md update

Add to the consumer commands section:

```markdown
- `/sdlc-ponder [slug]` — open the ideation workspace for exploring and committing ideas
- `/sdlc-ponder-commit <slug>` — crystallize a pondered idea into milestones and features
- `/sdlc-recruit <role>` — recruit an expert thought partner as a persistent agent
- `/sdlc-empathy <subject>` — deep user perspective interviews before decisions
```

---

## Integration With Existing Commands

### `/sdlc-plan`

No changes needed. `/sdlc-ponder-commit` produces a `plan.md` in the scrapbook and then invokes the existing `/sdlc-plan` flow. The plan format is already well-defined.

### `/sdlc-pressure-test`

After `/sdlc-ponder-commit` creates milestones, the natural next step is `/sdlc-pressure-test <milestone-slug>`. The pressure test reads the milestone vision and features — all of which were synthesized from the scrapbook's user perspectives and problem framing. The scrapbook provides richer input to the pressure test because the idea was already explored with thought partners.

### `/sdlc-status`

Should be aware of ponder entries. When showing project status, include a "Pondering" section listing active ponder entries.

### `/sdlc-specialize`

No changes. The agents recruited during ponder sessions are created via `/sdlc-recruit` and live in `.claude/agents/`. They're available project-wide, not just within the ponder context. `/sdlc-specialize` generates implementation-focused agents; ponder recruits ideation-focused thought partners. Different concerns.

### `sdlc focus`

No changes. Focus returns the highest-priority actionable feature directive. Ponder entries are not features and don't participate in the focus system. They're pre-feature ideation.

---

## Implementation Order

### Week 1: Core storage, CLI, and server — DONE

1. ~~Add `PonderStatus`, `PonderEntry`, `PonderTeamMember`, `PonderArtifactMeta` types to `sdlc-core`~~ ✓
2. ~~Add path helpers to `paths.rs`~~ ✓
3. ~~Implement `ponder.rs` module in `sdlc-core` (CRUD operations)~~ ✓
4. ~~Add `active_ponders` to state.yaml (with migration)~~ ✓
5. ~~Add `ponder` subcommand to `sdlc-cli`~~ ✓
6. ~~Tests for all core operations~~ ✓ (10 unit tests + 3 integration tests)
7. ~~Add `roadmap.rs` route module~~ ✓ (5 handlers)
8. ~~Register routes in `lib.rs`~~ ✓
9. ~~Update SSE polling to include `.sdlc/roadmap/`~~ ✓
10. ~~Add `PonderIndex` to search module~~ ✓
11. ~~Add error mapping for `PonderNotFound`/`PonderExists`~~ ✓

All 271 tests passing, clippy clean, zero warnings.

### Week 2: Command templates and frontend — DONE

12. ~~Write `SDLC_PONDER_COMMAND` / playbook / skill templates~~ ✓
13. ~~Write `SDLC_PONDER_COMMIT_COMMAND` / playbook / skill templates~~ ✓
14. ~~Write `SDLC_RECRUIT_COMMAND` / playbook / skill templates~~ ✓
15. ~~Write `SDLC_EMPATHY_COMMAND` / playbook / skill templates~~ ✓
16. ~~Register all in `init.rs` across all four platforms~~ ✓ (4 commands × 3 variants = 12 const strings)
17. ~~Update AGENTS.md template~~ ✓
18. ~~Add to `migrate_legacy_project_scaffolding()`~~ ✓
19. ~~Add TypeScript types~~ ✓ (`PonderStatus`, `PonderSummary`, `PonderTeamMember`, `PonderArtifact`, `PonderDetail`)
20. ~~Add API client methods~~ ✓ (5 methods: getRoadmap, getPonderEntry, createPonderEntry, updatePonderEntry, capturePonderArtifact)
21. ~~Create `RoadmapPage` component~~ ✓ (grid with status filter tabs)
22. ~~Create `PonderDetail` component~~ ✓ (two-column: scrapbook + team/meta)
23. ~~Add Roadmap to sidebar navigation~~ ✓ (Lightbulb icon between Milestones and Archive)
24. ~~Add pondering section to Dashboard~~ ✓ (active entries with copy buttons)
25. ~~Wire up SSE refresh for roadmap data~~ ✓ (useSSE on both pages)
26. ~~Add ponder status colors to StatusBadge~~ ✓ (exploring=violet, converging=amber, committed=emerald, parked=neutral)

All 271 tests passing, clippy clean.

### Week 3: Search integration, onboarding UI, and polish — DONE

27. ~~Add ponder entries to search query route (`GET /api/query/search`)~~ ✓ — merged `ponder_results` into response alongside feature `results`
28. ~~Update `SearchModal` to handle ponder results~~ ✓ — navigates to `/roadmap/:slug` for ponder matches, shows Lightbulb icon + status badge
29. ~~Add `useSearch` hook support for mixed result types~~ ✓ — `SearchResultItem` with `kind: 'feature' | 'ponder'`, merged and sorted by score
30. ~~Create "New Idea" onboarding flow on RoadmapPage~~ ✓ — inline creation form with auto-slug from title, brief text area, validation
31. ~~Add "New Idea" quick-action to Dashboard~~ ✓ — always-visible Pondering section with "View all →" link; empty state links to /roadmap
32. ~~Add ponder entries to `/sdlc-status` command output~~ ✓ — `sdlc ponder list` added to all 3 variants (COMMAND/PLAYBOOK/SKILL)
33. ~~Add tag management to PonderDetail~~ ✓ — `TagEditor` component with add/remove; `set_tags()` method on PonderEntry for full replacement; server `PUT` handler updated
34. ~~Add scrapbook inline capture form to PonderDetail~~ ✓ — `CaptureForm` component with filename + content + expandable UI
35. E2E smoke test deferred to manual verification

All 271 tests passing, clippy clean.

---

## What This Does NOT Include

- **No classifier for ponder entries.** There are no rules, no priority ordering, no deterministic "next action." Ideation is human-driven, not machine-driven.
- **No approval gates on scrapbook artifacts.** Artifacts are working documents. There's no Draft/Approved lifecycle. They exist or they don't.
- **No enforced artifact sequence.** You can write `exploration.md` before `problem.md`. The system tracks what exists and the `/sdlc-ponder` command suggests what's missing, but nothing is blocked.
- **No direct integration with the feature lifecycle.** Ponder entries feed the state machine via `/sdlc-ponder-commit` → `/sdlc-plan`. They don't become features themselves. The boundary is clean: scrapbook → plan → state machine.
- **No ponder-specific quality scores.** Quality scoring belongs to the feature lifecycle. Ideation quality is assessed by the thought partners during the session, not by a metric.
