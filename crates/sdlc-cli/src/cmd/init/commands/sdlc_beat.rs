use crate::cmd::init::registry::CommandDef;

const SDLC_BEAT_COMMAND: &str = r#"---
description: Take a beat — step back with a senior leadership lens and ask if we're building the right thing, in the right direction, for the right reasons
argument-hint: [domain | feature:<slug> | --week]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-beat

Step back. Read the project's actual vision. Trace what's live and what's planned.
Ask the hard question: *given where we said we were going, is this the right direction?*

This is a CPO taking a beat before the next quarter. A CTO reviewing the infrastructure
story. A lead asking "did we do the right thing?" — not "did we finish the tasks?"

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Arguments

- **No args** — broad project evaluation against `VISION.md`
- **`<domain>`** — scoped evaluation: `deployments`, `auth`, `fleet`, `notifications`, etc.
- **`feature:<slug>`** — single feature deep dive
- **`--week`** — generate (or refresh) the top-5 weekly check-in list from accumulated beat state

---

## Steps

### 1. Read VISION.md and project state

```bash
cat VISION.md 2>/dev/null || cat docs/vision.md 2>/dev/null || echo "no VISION.md found"
sdlc state
sdlc feature list --json
sdlc milestone list --json
sdlc ponder list --json 2>/dev/null || true
cat .sdlc/beat.yaml 2>/dev/null || echo "no beat history yet"
```

The VISION.md is the yardstick. Everything is evaluated against *this project's declared direction*,
not generic best practices. If there's no VISION.md, note it as a finding and infer direction
from milestones and feature history.

### 2. Determine the right leadership lens

Based on $ARGUMENTS (or the full project if no args), decide which lens applies:

| Domain | Lens |
|---|---|
| deployments, infra, cluster, fleet, CI/CD | CTO / Platform Lead |
| product features, UX, user flows | CPO / Product Lead |
| data model, APIs, architecture | Principal Engineer |
| cost, scale, reliability | CTO / SRE Lead |
| full project (no args) | CPO primary, CTO secondary |

### 3. Recruit the lens agent if missing

Check if the right agent exists in `.claude/agents/`:

```bash
ls .claude/agents/ 2>/dev/null
```

If the appropriate agent is **missing**, recruit it now using `/sdlc-recruit`:

```
/sdlc-recruit <lens role> --context "<domain focus>"
```

The recruited agent persists permanently — it will accumulate project health context
across repeated `sdlc-beat` calls. Do NOT create a throwaway agent.

If the agent **exists**, read it and continue with that persona's perspective.

### 4. Trace features and milestones in scope

For the domain/feature in $ARGUMENTS, read:
- All features touching this domain (check slugs, specs, designs for domain keywords)
- Their current phases: draft → ... → released
- Milestones that contain or depend on these features
- Any roadmap ponder entries (`sdlc ponder list`) related to this domain
- Recent completions: what shipped in the last 2-4 weeks?
- In-flight: what's currently in implementation, review, or audit?
- Planned: what's in draft/specified/planned phase?

Use `sdlc feature list --json` and read individual feature specs/designs for context.

### 5. Evaluate against VISION.md

Apply the leadership lens to the traced state. Ask:

**Retrospective (what we did):**
- Does what shipped actually move toward the declared vision?
- Were there scope or direction decisions that look questionable in hindsight?
- Are there signals of feature-level drift — things built that don't serve the vision?

**Current (what we're building):**
- Is in-flight work clearly connected to vision progress?
- Are there features stalled that are blocking critical vision outcomes?
- Any technical debt accumulating that will create a ceiling?

**Prospective (what's planned):**
- Does the planned work sequence make sense given the vision?
- Are there obvious gaps — things the vision needs that aren't in the pipeline?
- Is there work in the pipeline that should be deprioritized or cut?

### 6. Produce a verdict

Write a clear, direct assessment using the leadership persona's voice. Be specific.
Reference actual feature slugs, milestone names, and VISION.md language.

**Verdict format:**

```
## Beat — [scope] — [date]
**Lens:** [persona name + role]
**Verdict:** on-track | drifting | off-course

### What's working
[1-3 specific things that are clearly moving toward the vision]

### Concerns
[1-3 specific things that are misaligned, stalled, or missing — with slugs/names]

### Recommendation
[1-3 concrete, actionable steps — be direct, not diplomatic]
```

### 7. Write to .sdlc/beat.yaml

Update the beat state file. Schema:

```yaml
last_updated: "2026-03-03"
evaluations:
  - date: "2026-03-03"
    scope: project           # project | <domain> | feature:<slug>
    lens: "Kai Tanaka (CTO)" # agent name + role
    verdict: drifting        # on-track | drifting | off-course
    summary: "One sentence capturing the core finding"
    concerns:
      - slug: fleet-automation     # feature/milestone slug if applicable
        title: "Automation pipeline stalled 3 weeks"
        severity: high             # high | medium | low
        last_checked: "2026-03-03"
        trend: null                # null | improving | stalling | worsening
weekly:
  generated: "2026-03-03"
  items:
    - id: fleet-automation
      title: "Fleet automation pipeline"
      domain: deployments
      severity: high
      last_checked: null
      verdict: null
      trend: null
```

Rules:
- `evaluations` is append-only — never overwrite history
- `weekly.items` is replaced each time `--week` is run
- `concerns` entries carry forward across evaluations — update `trend` when re-checking the same slug
- Set `trend` based on: was this concern present last time? Is the situation better, same, or worse?

### 8. Handle --week mode

If $ARGUMENTS contains `--week`:

1. Read previous `weekly.items` from `.sdlc/beat.yaml` (if any)
2. Run a broad project evaluation (Steps 1–6 above)
3. Identify the top 5 things most worth checking in on this week, prioritized by:
   - Severity (high first)
   - Stall duration (how long has something been stuck)
   - Vision criticality (is this blocking a core vision outcome)
   - Trend (worsening items before stable ones)
4. Write the new `weekly` section to `.sdlc/beat.yaml`
5. Output the weekly check-in list:

```
## This Week's Beat — [date]

Top 5 things to check in on:

1. **[title]** (`[slug]`) — [one sentence why this week]
   Run: `/sdlc-beat feature:<slug>`

2. **[title]** ...
```

---

### Next

| Context | Next |
|---|---|
| Broad project | `**Next:** /sdlc-beat <top-concern-domain>` — drill into the biggest concern |
| Domain | `**Next:** /sdlc-beat feature:<slug>` — drill into the specific feature |
| Feature deep dive | `**Next:** /sdlc-ponder <slug>` — open a ponder session if realignment is needed |
| Weekly | `**Next:** /sdlc-beat <item-1-slug>` — start working through the list |
"#;

const SDLC_BEAT_PLAYBOOK: &str = r#"# sdlc-beat

Take a beat. Step back with a senior leadership lens and ask: given our vision, are we building the right thing in the right direction?

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Read VISION.md, `.sdlc/beat.yaml`, `sdlc state`, feature list, milestone list.
2. Determine the right leadership lens from $ARGUMENTS: CTO (infra/deployments), CPO (product/UX), Principal (architecture). Full project = CPO primary + CTO secondary.
3. Check `.claude/agents/` — if the lens agent is missing, recruit it permanently via `/sdlc-recruit`.
4. Trace features/milestones in scope across all phases (shipped, in-flight, planned). Read specs and designs.
5. Evaluate against VISION.md: retrospective (did we build right?), current (is in-flight aligned?), prospective (are gaps covered?).
6. Produce verdict: on-track | drifting | off-course. Be direct, reference slugs and vision language.
7. Write verdict + concerns to `.sdlc/beat.yaml` (append evaluations; carry forward concern trends).
8. --week mode: read previous weekly items, run broad evaluation, output top-5 prioritized check-in list, write new `weekly` section to beat.yaml.
9. **Next:** drill into top concern, open ponder session, or start weekly check-in list.
"#;

const SDLC_BEAT_SKILL: &str = r#"---
name: sdlc-beat
description: Take a beat — step back with a senior leadership lens to ask if the project is building the right thing in the right direction. Use for broad project review, domain-scoped evaluation, single feature deep dive, or weekly check-in prioritization.
---

# SDLC Beat Skill

Step back. Evaluate direction, not just completion.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Modes

- **No args** — broad project evaluation
- **`<domain>`** — domain-scoped: deployments, auth, fleet, etc.
- **`feature:<slug>`** — single feature deep dive
- **`--week`** — generate top-5 weekly check-in list

## Workflow

1. Read VISION.md, `.sdlc/beat.yaml` (history), `sdlc state`, feature + milestone lists.
2. Determine lens: CTO (infra), CPO (product), Principal (architecture). Full project = both.
3. Check `.claude/agents/` for the lens agent. If missing: recruit permanently via `/sdlc-recruit`.
4. Trace features/milestones in scope across all phases. Read specs and designs.
5. Evaluate against VISION.md: retrospective + current + prospective.
6. Produce verdict (on-track | drifting | off-course) with specific concerns and recommendations.
7. Write to `.sdlc/beat.yaml` — append evaluations, carry forward concern trends.
8. For `--week`: output top-5 prioritized check-in items, write `weekly` section to beat.yaml.
9. **Next:** drill into top concern or open ponder session for realignment.
"#;

pub static SDLC_BEAT: CommandDef = CommandDef {
    slug: "sdlc-beat",
    claude_content: SDLC_BEAT_COMMAND,
    gemini_description: "Step back with a leadership lens — evaluate if we're building the right thing in the right direction",
    playbook: SDLC_BEAT_PLAYBOOK,
    opencode_description: "Step back with a leadership lens — evaluate if we're building the right thing in the right direction",
    opencode_hint: "[domain | feature:<slug> | --week]",
    skill: SDLC_BEAT_SKILL,
};
