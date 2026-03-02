use crate::cmd::init::registry::CommandDef;

const SDLC_SUGGEST_COMMAND: &str = r#"---
description: Suggest what to ponder next — analyzes project state using a maturity ladder and advisory history to recommend the right next thing
argument-hint: (no arguments)
allowed-tools: Bash, Read, Glob, Grep
---

# sdlc-suggest

Analyze the project state — using advisory history and the maturity ladder — and recommend
3-5 specific, actionable ponder topics. Work through basics before roadmap. Use history
to avoid re-scanning what you already know.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Steps

### 1. Read advisory history and project state

```bash
cat .sdlc/advisory.yaml 2>/dev/null || echo "no advisory history"
sdlc feature list --json
sdlc milestone list --json
sdlc ponder list --json
```

Also read (if they exist): `VISION.md`, `ARCHITECTURE.md`

The advisory history tells you:
- What stage the project was at last time
- What findings were open, addressed, or dismissed
- When the last analysis was run and how many files the project had

Use this to decide: can you reuse previous findings, or does the project state suggest a fresh look?

### 2. Orient to the maturity ladder

Work through stages in order. Only advance to the next stage when the current one is clean enough.
Use your judgement — don't be pedantic, but don't skip a broken foundation to jump to roadmap ideas.

| Stage | What to check |
|---|---|
| **Health** | Does it build? Do tests pass? Is there dead code or circular dependencies? Lint clean? |
| **Consistency** | One logging pattern, not three? Config access DRY? Error shapes consistent? Naming uniform? |
| **Refactor** | Duplicated logic extracted? Files/functions over threshold broken up? Missing abstractions? |
| **Structure** | Common UI components identified and DRY'd? Module boundaries respected? No architectural drift? |
| **Roadmap** | Obvious gaps in the feature set? Near-term user-facing improvements? Unfinished work visible? |
| **Advanced** | Strategic bets, ecosystem integrations, speculative improvements? |

**Decision logic (your call):**
- If health findings exist and are recent → start there, don't skip to roadmap
- If history is old (>2 weeks) or project has grown significantly → re-check current stage
- If a stage is clean → move to the next one without dwelling
- If history shows a finding was dismissed/wont-fix → don't resurface it

### 3. Run stage-appropriate checks

Read files. Look for the patterns that belong to the current stage. Don't re-examine stages
the history shows are already clean.

### 4. Write findings back to advisory history

Update `.sdlc/advisory.yaml` with your assessment. The schema:

```yaml
runs:
  - run_at: "2026-02-28T12:00:00Z"    # ISO 8601
    file_count: 42                      # optional: approximate file count
    stage_reached: health               # health | consistency | refactor | structure | roadmap | advanced
    summary: "One sentence describing what you found at this stage"
findings:
  - id: adv-a1b2c3                     # adv- + 6 alphanum chars (stable across runs)
    stage: health
    title: "Short title (5-8 words)"
    description: "Specific finding with file reference if applicable"
    status: open                        # open | acknowledged | resolved | dismissed
    created_at: "2026-02-28T12:00:00Z"
    resolved_at: null                   # omit or set when resolved
```

Create the file if it doesn't exist. Append to `runs` (never overwrite history).
For `findings`, merge — not replace:
- If a previous finding is no longer present in the code → mark it `resolved`
- Add new findings with `open` status
- Preserve findings marked `acknowledged` or `dismissed` — do not re-open them

### 5. Present suggestions

Based on where the project is in the maturity ladder, return 3-5 suggestions:

---

## Suggested Ponder Topics

**Current stage:** [Stage name] — [one-line reason why you're at this stage]

### 1. [Title]
**Slug:** `slug-here`
**Why now:** One sentence — why this is the right next thing given the project state and advisory history.
**What to explore:** 2-3 sentences on what the ponder session would investigate.

---

### 6. Next

**Next:** `/sdlc-ponder <suggested-slug>` — pick the most compelling one and start exploring
"#;

const SDLC_SUGGEST_PLAYBOOK: &str = "\
Analyze the project state using the maturity ladder and advisory history. Work through basics before roadmap.\n\
\n\
Steps:\n\
1. Read: `.sdlc/advisory.yaml`, `sdlc feature list --json`, `sdlc milestone list --json`, `sdlc ponder list --json`, VISION.md, ARCHITECTURE.md\n\
2. Orient to the maturity ladder: Health → Consistency → Refactor → Structure → Roadmap → Advanced\n\
3. Start at the current stage (from advisory history). Check if previous findings still apply.\n\
4. Run stage-appropriate analysis (read files; look for the patterns that belong to this stage).\n\
5. Write findings back to `.sdlc/advisory.yaml`. Schema: top-level `runs` (append-only) and `findings` (merged). Status values: open | acknowledged | resolved | dismissed.\n\
6. Return 3-5 suggestions scoped to where the project actually is. Show current stage.\n\
7. End with **Next:** `/sdlc-ponder <slug>`\n\
";

const SDLC_SUGGEST_SKILL: &str = r#"---
name: sdlc-suggest
description: Suggest what to ponder next. Uses the maturity ladder and advisory history to recommend the right next thing. Use when starting a new project or unsure what to explore.
---

# SDLC Suggest Skill

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Read `.sdlc/advisory.yaml` (may not exist yet). Read project state: `sdlc feature list --json`, milestones, ponders, VISION.md, ARCHITECTURE.md.
2. Orient to the maturity ladder: **Health → Consistency → Refactor → Structure → Roadmap → Advanced**. Start at the stage from advisory history; if no history, start at Health.
3. Use advisory history to skip what's already clean. Re-check open findings. Mark addressed findings.
4. Run stage-appropriate analysis: read files, look for patterns specific to that stage. You decide the depth based on how old the history is and how much the project has changed.
5. Write findings back to `.sdlc/advisory.yaml` (append, never overwrite history).
6. Present 3-5 suggestions scoped to the current stage. Show stage name in the output.
7. **Next:** `/sdlc-ponder <suggested-slug>`
"#;

pub static SDLC_SUGGEST: CommandDef = CommandDef {
    slug: "sdlc-suggest",
    claude_content: SDLC_SUGGEST_COMMAND,
    gemini_description: "Suggest what to ponder next based on current project state",
    playbook: SDLC_SUGGEST_PLAYBOOK,
    opencode_description: "Suggest what to ponder next based on current project state",
    opencode_hint: "",
    skill: SDLC_SUGGEST_SKILL,
};
