use crate::cmd::init::registry::CommandDef;

const SDLC_GUIDELINE_COMMAND: &str = r#"---
description: Build an evidence-backed guideline through perspective research and TOC-first distillation. Researches from five lenses (including web search for prior art), constructs a Table of Contents, then distills each section from real codebase evidence.
argument-hint: <slug-or-problem-description>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, WebSearch, WebFetch, Task, AskUserQuestion
---

# sdlc-guideline

Build a guideline from evidence, not opinion. Five research perspectives feed a Table of Contents.
The TOC is the structural contract — it commits to scope and ordering before any writing happens.
Then each section is distilled from the evidence, not invented.

Phase sequence: **problem → agenda → perspectives → toc → distillation → publish**

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

---

## Entering the workspace

### If $ARGUMENTS is an existing slug

```bash
sdlc investigate show <slug>
sdlc investigate session list <slug>
```

Read the manifest and all artifacts. If sessions exist, read the most recent:

```bash
sdlc investigate session read <slug> <N>
```

Orient from the orientation strip (WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL).
Summarize what phase we're in, what's been captured, and what comes next. Then run the next phase.

### If $ARGUMENTS describes a new problem

1. Derive a slug (lowercase, hyphens, max 40 chars).
2. Create the investigation:
```bash
sdlc investigate create <slug> --title "<problem title>" --kind guideline \
  --context "<brief problem description>"
sdlc investigate update <slug> --scope "<where this applies: language/layer/subsystem>"
```

---

## Phase: problem

**Goal:** Frame the recurring problem with precision.

Capture the initial problem framing:

```bash
sdlc investigate capture <slug> --content "<markdown>" --as problem.md
```

`problem.md` must answer:
- **Scope:** where does this apply? (language, layer, subsystem, team)
- **Pattern:** what is the recurring behavior that causes harm?
- **Harm:** what goes wrong when this pattern appears? What's the blast radius?
- **Current state:** is this partially addressed? Are there existing conventions?

After capturing, record the problem statement:
```bash
sdlc investigate update <slug> --problem-statement "<one-line: the pattern and why it matters>"
sdlc investigate update <slug> --phase agenda
```

---

## Phase: agenda

**Goal:** Define 4-6 questions the guideline must answer.

Read `problem.md`. Derive targeted research questions. These anchor the entire investigation —
specific enough to drive evidence gathering, not generic.

```bash
sdlc investigate capture <slug> --content "<markdown>" --as research-agenda.md
```

`research-agenda.md` format:
```markdown
# Research Agenda: <slug>

## Questions
1. <question> — why this matters to the final guideline
2. <question> — why this matters
...

## Not in scope
- <what we are explicitly not researching and why>
```

Advance:
```bash
sdlc investigate update <slug> --phase perspectives
```

---

## Phase: perspectives

**Goal:** Gather evidence from five distinct research lenses.

Run all five lenses in sequence. Each writes a separate evidence artifact.
Evidence quality determines guideline quality — do not rush.

### Lens 1: Anti-pattern Archaeologist

Search the codebase for real occurrences of the problem. Use Grep, Glob, Read.
Cite actual file:line references. Note the context — why did it happen here?

```bash
sdlc investigate capture <slug> --content "<markdown>" --as evidence-antipatterns.md
```

Format:
```markdown
# Anti-patterns Found

## [Instance title]
**File:** `path/to/file.rs:42`
**Pattern:** <what is wrong>
**Why:** <why this is the problematic form>
```code snippet```

## Summary
Found N instances across M files. Most concentrated in: <areas>.
```

### Lens 2: Exemplar Scout

Find the best existing implementations of the right approach in this codebase.

```bash
sdlc investigate capture <slug> --content "<markdown>" --as evidence-exemplars.md
```

Same format — positive examples with file:line citations. What does doing it right look like here?

### Lens 3: Prior Art Mapper

Research how the broader community addresses this problem.
**Use WebSearch** to find named patterns, RFC language, style guide conventions, linter rules,
and how similar projects handle the same tradeoff.

```bash
sdlc investigate capture <slug> --content "<markdown>" --as evidence-priorart.md
```

Label each source: `[from training knowledge]` or `[from web: URL]`.

### Lens 4: Adjacent Pattern Analyst

Find related patterns that interact with this one. What else breaks or benefits when this
guideline is followed? What adjacent rules exist that developers already know?

```bash
sdlc investigate capture <slug> --content "<markdown>" --as evidence-adjacent.md
```

### Lens 5: Impact Assessor

Estimate blast radius and priority:
- How many occurrences exist right now?
- Which areas are most exposed?
- Typical failure mode when violated?
- Is this "always fails" or "occasionally fails"?

```bash
sdlc investigate capture <slug> --content "<markdown>" --as evidence-impact.md
```

Advance after all five lenses:
```bash
sdlc investigate update <slug> --phase toc
```

---

## Phase: toc

**Goal:** Commit to structure before writing.

Read ALL five evidence artifacts. Group findings by theme. Draft a Table of Contents that:
- Starts with the core rule (the one thing someone must never/always do)
- Orders sections by importance — most evidence density first
- Includes edge cases and exceptions after the main rules
- Ends with enforcement and migration guidance

```bash
sdlc investigate capture <slug> --content "<markdown>" --as toc.md
```

`toc.md` format:
```markdown
# Guideline TOC: <slug>

## Table of Contents
1. **<Section title>** — <one-line: what rule or principle this section establishes>
2. **<Section title>** — <one-line description>
...

## Rationale
<Why section 1 is first. What groupings emerged. What was left out and why.>

## Evidence Summary
- Anti-patterns found: N
- Good examples found: M
- Prior art sources: P
- Adjacent patterns: Q
```

The TOC is the structural contract. Once captured, it defines exactly what gets written.

Advance:
```bash
sdlc investigate update <slug> --phase distillation
```

---

## Phase: distillation

**Goal:** Write each TOC section from evidence, not from opinion.

Read `toc.md` to know the sections. Read the relevant evidence artifacts for each.
Distill what the evidence shows — do not invent rules.

**Distillation principles:**
- Every rule must be backed by at least one real example (file:line) from this project
- Rules from prior art are labeled `[community practice]`
- If evidence is thin for a section, say so — write what you know, note the gap
- Use `⚑ Rule:` for the actual rule statement
- Use `✓ Good:` and `✗ Bad:` for concrete examples

Write the full guideline as `guideline-draft.md`:

```bash
sdlc investigate capture <slug> --content "<full markdown>" --as guideline-draft.md
```

`guideline-draft.md` structure:
```markdown
# <Guideline Title>

**Scope:** <where this applies>
**Problem:** <one-line problem statement>
**Confidence:** <High | Medium — based on evidence density>

---

## 1. [First TOC section]

⚑ **Rule:** <the actual rule>

**Why:** <evidence-backed explanation — cite the impact assessor findings>

✗ **Bad:** `path/to/file.rs:42` — <why this is wrong>
```bad snippet```

✓ **Good:** `path/to/file.rs:87` — <why this is right>
```good snippet```

---

## 2. [Second section — repeat pattern]

---

## Enforcement

[Grep patterns, lint rules, or PR checklist items that catch violations]

## Migration

[How to address existing violations — scope estimate, remediation steps]
```

After capturing:
```bash
sdlc investigate update <slug> --principles-count <N-sections>
sdlc investigate update <slug> --phase publish
```

---

## Phase: publish

**Goal:** Finalize and publish.

Read `guideline-draft.md`. Review:
- All TOC sections present and backed by evidence
- Enforcement section complete
- Migration guidance actionable

Publish to the project guidelines directory:
```bash
mkdir -p .sdlc/guidelines
```

Write the final guideline to `.sdlc/guidelines/<slug>.md`.

Update the manifest:
```bash
sdlc investigate update <slug> --output-ref ".sdlc/guidelines/<slug>.md"
sdlc investigate update <slug> --status complete
```

Update the guidelines index at `.sdlc/guidelines/index.yaml`. Read the file if it exists,
add or update the entry for this guideline, and write it back:
```yaml
guidelines:
  - slug: <slug>
    title: "<title from manifest>"
    scope: "<guideline_scope from manifest>"
    path: .sdlc/guidelines/<slug>.md
    created: <today's date>
```
Preserve all existing entries. If the slug already exists in the index, update it in place.

If violations require remediation, create a tracking feature:
```bash
sdlc feature create guideline-<slug>-enforcement --title "Enforce: <guideline title>"
```

---

## Session logging (MANDATORY)

Every session MUST end with:
1. Write the complete session Markdown to `/tmp/guideline-session-<slug>.md` using the Write tool
2. Run: `sdlc investigate session log <slug> --file /tmp/guideline-session-<slug>.md`

Session frontmatter:
```yaml
---
session: <N>
timestamp: <ISO-8601 UTC>
orientation:
  current: "<phase we're in and what was accomplished this session>"
  next: "<concrete next action>"
  commit: "<condition that unlocks the next phase>"
---
```

---

## Phase transition reference

| Phase | Gate artifact | Command to advance |
|---|---|---|
| problem | `problem.md` | `sdlc investigate update <slug> --phase agenda` |
| agenda | `research-agenda.md` | `sdlc investigate update <slug> --phase perspectives` |
| perspectives | all 5 `evidence-*.md` files | `sdlc investigate update <slug> --phase toc` |
| toc | `toc.md` | `sdlc investigate update <slug> --phase distillation` |
| distillation | `guideline-draft.md` | `sdlc investigate update <slug> --phase publish` |
| publish | `.sdlc/guidelines/<slug>.md` | `sdlc investigate update <slug> --status complete` |

---

## Next steps by state

| State | Next |
|---|---|
| New slug created | `**Next:** /sdlc-guideline <slug>` (run problem phase) |
| problem phase complete | `**Next:** /sdlc-guideline <slug>` (run agenda phase) |
| agenda written | `**Next:** /sdlc-guideline <slug>` (run perspectives — all 5 lenses) |
| perspectives complete | `**Next:** /sdlc-guideline <slug>` (run toc phase) |
| toc written | `**Next:** /sdlc-guideline <slug>` (run distillation) |
| draft complete | `**Next:** /sdlc-guideline <slug>` (run publish) |
| published | `**Next:** /sdlc-ponder` or `/sdlc-run <feature-slug>` |
"#;

const SDLC_GUIDELINE_PLAYBOOK: &str = r#"# sdlc-guideline

Build an evidence-backed guideline through five research perspectives and TOC-first distillation.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Phase sequence

`problem → agenda → perspectives → toc → distillation → publish`

## Steps

1. **Load or create:** If slug exists, `sdlc investigate show <slug>` + read sessions/artifacts. If new, `sdlc investigate create <slug> --kind guideline --title "..." --context "..."`.
2. **problem phase:** Write `problem.md` (scope, pattern, harm, current state). `sdlc investigate update <slug> --problem-statement "..." --phase agenda`.
3. **agenda phase:** Derive 4-6 research questions. Write `research-agenda.md`. `sdlc investigate update <slug> --phase perspectives`.
4. **perspectives phase — run all 5 lenses:**
   - Lens 1 (Anti-pattern Archaeologist): grep codebase, write `evidence-antipatterns.md` with file:line citations.
   - Lens 2 (Exemplar Scout): find good implementations, write `evidence-exemplars.md` with file:line citations.
   - Lens 3 (Prior Art Mapper): research community knowledge and named patterns — use WebSearch — write `evidence-priorart.md`.
   - Lens 4 (Adjacent Pattern Analyst): find related patterns, write `evidence-adjacent.md`.
   - Lens 5 (Impact Assessor): estimate frequency and blast radius, write `evidence-impact.md`.
   - `sdlc investigate update <slug> --phase toc`.
5. **toc phase:** Read all evidence artifacts. Group by theme. Order by importance. Write `toc.md` (numbered sections, rationale, evidence summary). `sdlc investigate update <slug> --phase distillation`.
6. **distillation phase:** For each TOC section, read relevant evidence and distill. Write `guideline-draft.md` with `⚑ Rule:`, `✓ Good:`, `✗ Bad:` markers and file:line citations. `sdlc investigate update <slug> --principles-count <N> --phase publish`.
7. **publish phase:** Write final guideline to `.sdlc/guidelines/<slug>.md`. `sdlc investigate update <slug> --output-ref ".sdlc/guidelines/<slug>.md" --status complete`. Read `.sdlc/guidelines/index.yaml` (or create it), add/update this entry (`slug`, `title`, `scope`, `path`, `created`), write it back — preserving all other entries.
8. **Session log (MANDATORY):** Write session to `/tmp/guideline-session-<slug>.md`, then `sdlc investigate session log <slug> --file /tmp/guideline-session-<slug>.md`.
9. **Next:** `/sdlc-guideline <slug>` to continue, or report completion.
"#;

const SDLC_GUIDELINE_SKILL: &str = r#"---
name: sdlc-guideline
description: Build an evidence-backed guideline through perspective research and TOC-first distillation. Researches from five lenses (including web search for prior art), constructs a Table of Contents, then distills each section from real codebase evidence.
---

# SDLC Guideline Skill

Build a guideline from evidence, not opinion.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Phase sequence

`problem → agenda → perspectives → toc → distillation → publish`

## Workflow

1. Load or create investigation: `sdlc investigate show <slug>` or `sdlc investigate create <slug> --kind guideline`.
2. Read all sessions and artifacts to orient.
3. Run the current phase to completion, writing the gate artifact.
4. Advance: `sdlc investigate update <slug> --phase <next>`.
5. Five perspectives in the `perspectives` phase: anti-patterns (codebase grep), exemplars (good examples), prior art (community knowledge + web search), adjacent patterns, impact assessment.
6. TOC (`toc.md`) commits to structure before writing — ordered by evidence density.
7. Distillation writes each TOC section from evidence — every rule cites a real file:line.
8. Publish to `.sdlc/guidelines/<slug>.md`. Update manifest with output-ref and status complete. Update `.sdlc/guidelines/index.yaml` — read existing (or create), add/update entry (`slug`, `title`, `scope`, `path`, `created`), write back preserving all other entries.
9. Log session (MANDATORY): Write to `/tmp/guideline-session-<slug>.md`, then `sdlc investigate session log <slug> --file /tmp/...`.
10. **Next:** `/sdlc-guideline <slug>` to continue next phase.
"#;

pub static SDLC_GUIDELINE: CommandDef = CommandDef {
    slug: "sdlc-guideline",
    claude_content: SDLC_GUIDELINE_COMMAND,
    gemini_description:
        "Build an evidence-backed guideline through perspective research and TOC-first distillation",
    playbook: SDLC_GUIDELINE_PLAYBOOK,
    opencode_description:
        "Build an evidence-backed guideline through perspective research and TOC-first distillation",
    opencode_hint: "<slug-or-problem-description>",
    skill: SDLC_GUIDELINE_SKILL,
};
