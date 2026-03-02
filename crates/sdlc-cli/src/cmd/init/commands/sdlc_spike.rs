use crate::cmd::init::registry::CommandDef;

const SDLC_SPIKE_COMMAND: &str = r#"---
description: Time-boxed technical spike — research, prototype, validate, and report. Examines a reference project, searches for better alternatives, builds a working prototype in a temp workspace, and produces a findings report with specific implementation details.
argument-hint: <slug> — <need description>; [see <reference-url-or-repo>]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, WebSearch, WebFetch, Agent
---

# sdlc-spike

Execute a time-boxed technical spike: understand the need, explore the landscape, build a working prototype, validate it, and produce a concrete implementation recommendation. The output is not a document — it is a working prototype plus a decision record.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Parsing $ARGUMENTS

Arguments have the form: `<slug> — <need>; [see <url-or-repo>]`

- **slug** — identifier for this spike (workspace and report path)
- **need** — what we are trying to achieve (not a solution — a capability or problem)
- **reference** — optional seed project or URL; starting point, not a commitment

If no slug is given, derive one from the need (lowercase, hyphens, max 40 chars).

---

## Phase 1: Frame

**Understand the need precisely before touching any code.**

1. Restate the need in one sentence: "We need X so that Y."
2. Read project context:
   - `CLAUDE.md` — stack, conventions, patterns
   - `ARCHITECTURE.md` — system shape, key components
   - Any source files directly relevant to the need (grep for related terms)
3. Define success criteria: what does "it works" mean exactly?
   - What data is produced?
   - Who consumes it and how?
   - What does failure look like?
4. Identify constraints: language, embedded vs. external, privacy, performance.

Write a one-paragraph frame to `/tmp/spike-<slug>/frame.md`.

---

## Phase 2: Landscape

**Understand the solution space — do not commit to the first thing found.**

### Step A: Examine the reference (if provided)

```bash
gh repo clone <org/repo> /tmp/spike-<slug>/ref/<repo-name>
```

Read the source at the mechanism level:
- How does it actually work?
- What does it capture / emit / store?
- What is the integration surface (API, SDK, hooks, env vars)?
- What are the constraints (language, runtime, platform)?

### Step B: Search for alternatives

WebSearch using multiple angles:
- `"[need keyword] [language/stack] [year]"`
- `"[need keyword] embedded database alternative"`
- `"best [need keyword] open source [year]"`

For each candidate, evaluate:
| Candidate | Mechanism | Integration effort | Maintenance | Alignment to need |
|---|---|---|---|---|

### Step C: Select winner(s)

Pick 1-2 candidates to prototype. Document why others were eliminated. If the reference project is not the best fit, say so explicitly.

---

## Phase 3: Prototype

**Build a working integration in an isolated workspace.**

```bash
mkdir -p /tmp/spike-<slug>/prototype
```

The prototype must:
- Use the winner's actual API / SDK / hooks — no mocking
- Integrate with our actual data shapes, not invented ones
- Produce real output that answers the success criteria from Phase 1

Work iteratively:
1. Minimal integration — get it to run
2. Adapt to our use case — real data, real context
3. Handle the cases that matter — not just the happy path

If a candidate fails, document why and try the next.

---

## Phase 4: Validate

**Prove it works — capture real output.**

Run the prototype. Capture:
- Actual output (stdout, files produced, DB contents, etc.)
- Evidence the specific capability works (not "it ran" — "it captured X")
- Edge cases: concurrent runs, large payloads, crash behavior

Save captured output to `/tmp/spike-<slug>/prototype/validation-output.txt`.

If validation fails:
- Document the failure mode precisely
- Try an alternative candidate (back to Phase 3)
- If all candidates fail, the verdict is REJECT with root cause

---

## Phase 5: Report

**Write the decision record to `.sdlc/spikes/<slug>/findings.md`.**

```bash
mkdir -p .sdlc/spikes/<slug>
```

`findings.md` must contain:

```markdown
# Spike: <title>

**Slug:** <slug>
**Date:** <ISO date>
**Verdict:** ADOPT | ADAPT | REJECT

## The Question
<one sentence: what we needed to answer>

## Success Criteria
<what "it works" means — from Phase 1>

## Candidates Evaluated
| Candidate | Verdict | Reason |
|---|---|---|
| <ref project> | winner/eliminated | <why> |

## Winner: <project/approach>

### Why It Won
<mechanism, alignment to need, constraints satisfied>

### How It Works
<what the prototype proved — not documentation paraphrase>

### Working Prototype
Location: `/tmp/spike-<slug>/prototype/`
Key files: <list>

### Validation Evidence
<actual output captured from Phase 4>

## Implementation Plan

### Files to add
- `<path>` — <purpose>

### Files to modify
- `<path>:<function>` — <what changes and why>

### Dependencies to add
<exact package names + versions + how to add>

### Configuration
<env vars, config file changes, exact values>

### Integration points
<specific functions, traits, middleware in our codebase where this hooks in>

### Code patterns
<concrete code snippets — not pseudocode>

## Risks and Open Questions
- <risk>: <mitigation>

## What Was Not Tried
<candidates considered but not prototyped, and why>
```

---

## Step Back — Before Finalizing

Challenge:
- Does the prototype prove the specific thing we needed, or just something adjacent?
- Is the implementation plan specific enough to follow without re-researching?
- Are there risks in the winner that prototyping did not surface?
- Would adopting this create a new problem we have not considered?

If the prototype code is worth preserving, copy key files to `.sdlc/spikes/<slug>/prototype/` before ending.

---

**Next:**

| Verdict | Next |
|---|---|
| ADOPT | `**Next:** /sdlc-hypothetical-planning <feature-slug>` (plan the integration) |
| ADAPT | `**Next:** /sdlc-ponder <slug>` (shape what adaptation looks like) |
| REJECT | `**Next:** /sdlc-ponder <slug>` (explore alternative approaches) |
"#;

const SDLC_SPIKE_PLAYBOOK: &str = r#"# sdlc-spike

Execute a time-boxed technical spike: understand the need, explore the landscape, build a working prototype, validate it, and produce a concrete implementation recommendation.

> Read `.sdlc/guidance.md` for engineering principles.

## Argument format
`<slug> — <need description>; [see <reference-url-or-repo>]`

## Steps

1. **Frame** — restate the need precisely ("We need X so that Y"); read CLAUDE.md + ARCHITECTURE.md + relevant source; define success criteria; identify constraints; write `/tmp/spike-<slug>/frame.md`
2. **Examine reference** — `gh repo clone` the referenced project; read its mechanism, integration surface, and constraints at the source level
3. **Search alternatives** — WebSearch for other candidates; evaluate each on mechanism, integration effort, maintenance, and alignment; select 1-2 to prototype
4. **Prototype** — `mkdir /tmp/spike-<slug>/prototype`; build real integration using winner's actual API with our actual data shapes; iterate to working state; no mocking
5. **Validate** — run prototype; capture actual output to `validation-output.txt`; prove the specific capability works; test edge cases
6. **Report** — write `.sdlc/spikes/<slug>/findings.md` with: verdict (ADOPT/ADAPT/REJECT), candidates evaluated, winner rationale, validation evidence, and full implementation plan

## Implementation plan requirements
Specific enough to follow without re-researching:
- Exact file paths and functions to modify
- Exact package names and versions
- Exact config values and env vars
- Concrete code snippets (not pseudocode)
- Specific integration points in our codebase

**Next:**
| Verdict | Next |
|---|---|
| ADOPT | `/sdlc-hypothetical-planning <feature-slug>` |
| ADAPT | `/sdlc-ponder <slug>` |
| REJECT | `/sdlc-ponder <slug>` |
"#;

const SDLC_SPIKE_SKILL: &str = r#"---
name: sdlc-spike
description: Time-boxed technical spike — research a need, examine reference projects, search for better alternatives, build a working prototype in /tmp, validate it, and write a findings report with implementation details to .sdlc/spikes/<slug>/findings.md.
---

# SDLC Spike Skill

Execute a time-boxed technical spike with working prototype and decision record.

> Read `.sdlc/guidance.md` for engineering principles.

## Workflow

1. **Frame** — restate need precisely; read CLAUDE.md + ARCHITECTURE.md; define success criteria; write `/tmp/spike-<slug>/frame.md`
2. **Examine reference** — clone and read the referenced project's mechanism and integration surface
3. **Search alternatives** — WebSearch for better options; evaluate and select winner
4. **Prototype** — build real integration in `/tmp/spike-<slug>/prototype/` using winner's actual API with our actual data shapes; no mocking
5. **Validate** — run it; capture real output to `validation-output.txt`; prove the specific capability works
6. **Report** — write `.sdlc/spikes/<slug>/findings.md` with: verdict, candidates, winner rationale, validation evidence, and implementation plan (exact files, deps, config, code)

| Verdict | Next |
|---|---|
| ADOPT | `**Next:** /sdlc-hypothetical-planning <feature-slug>` |
| ADAPT | `**Next:** /sdlc-ponder <slug>` |
| REJECT | `**Next:** /sdlc-ponder <slug>` |
"#;

pub static SDLC_SPIKE: CommandDef = CommandDef {
    slug: "sdlc-spike",
    claude_content: SDLC_SPIKE_COMMAND,
    gemini_description: "Time-boxed technical spike — research, prototype, validate, and report with implementation details",
    playbook: SDLC_SPIKE_PLAYBOOK,
    opencode_description: "Time-boxed technical spike — research, prototype, validate, and report with implementation details",
    opencode_hint: "<slug> — <need description>; [see <reference-url-or-repo>]",
    skill: SDLC_SPIKE_SKILL,
};
