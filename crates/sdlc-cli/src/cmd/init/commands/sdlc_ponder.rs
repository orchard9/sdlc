use crate::cmd::init::registry::CommandDef;

const SDLC_PONDER_COMMAND: &str = r#"---
description: Open the ideation workspace — explore ideas with recruited thought partners, capture artifacts in the scrapbook, commit when ready. Embeds ideation, empathy, and recruitment protocols natively.
argument-hint: [slug or new idea description]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task, AskUserQuestion
---

# sdlc-ponder

Open the ponder workspace for creative exploration. This command sets the context —
from here, everything is conversation. You have access to all thinking tools. Artifacts
you produce land in the scrapbook and persist across sessions. Every session is logged
as a Markdown dialogue that accumulates over time.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Entering the workspace

### If $ARGUMENTS is a known slug

```bash
sdlc ponder show <slug>
sdlc ponder session list <slug>
```

Read the manifest, team, and all scrapbook artifacts. Load the team's agent definitions.
Read the session list — if sessions exist, read the most recent one to restore full context:

```bash
sdlc ponder session read <slug> <number>
```

Orient yourself from the **orientation strip** in the most recent session (or the manifest):

```
WHERE WE ARE   <current state of the thinking>
→ NEXT MOVE    <what the previous session said to do next>
COMMIT SIGNAL  <condition that unlocks commitment>
```

Summarize where the idea stands: what's been explored, who's on the team, open questions,
and what the orientation strip says to do next. Then dive in.

### If $ARGUMENTS looks like a new idea (not an existing slug)

1. Derive a slug from the idea text (lowercase, hyphens, max 40 chars).
2. Create the entry:
```bash
sdlc ponder create <slug> --title "<derived title>"
sdlc ponder capture <slug> --content "<verbatim user text>" --as brief.md
```
3. Read the brief. Identify domain signals. Recruit 2-3 initial thought partners —
   always include:
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
# Create the agent, then register them
sdlc ponder team add <slug> --name "<name>" --role "<role>" \
  --context "<context>" --agent .claude/agents/<name>.md
```

### Embedded capabilities

#### Ideation protocol

When exploring a problem:
1. **Understand** — capture the problem statement, your interpretation, scope, success criteria
2. **Gather context** — read relevant code, specs, adjacent systems
3. **Synthesize** — landscape, constraints, gaps, key files
4. **Consult thought partners** — channel each recruited expert's perspective
5. **Explore solutions** — 3-4 options including "do nothing", with trade-offs
6. **Step back** — assumption audit, fresh eyes test, skeptic's questions, reversal
7. **Think out loud** — share learnings, surprises, core tension, questions
8. **Collaborate** — listen, adjust, iterate with the user

#### Empathy protocol

When exploring user perspectives:
1. **Identify stakeholders** — direct users, indirect, blockers, forgotten
2. **Create perspective agents** — specific humans in specific situations
3. **Deep interview each** — context, needs, friction, delight, deal-breakers
4. **Synthesize** — alignments, conflicts, gaps, surprises
5. **Step back** — bias check, quiet voice, stress test, humility check
6. **Recommend** — evidence-based, tradeoffs acknowledged, unknowns flagged

Always include at least 3 perspectives. Always include a skeptic.

#### Recruitment protocol

When a domain signal emerges and you need a thought partner:
1. **Orient** — what expertise is needed and why
2. **Design the expert** — real name, career background at named companies, specific
   technical philosophy, strong opinions
3. **Create the agent** — write to `.claude/agents/<name>.md`
4. **Register** — `sdlc ponder team add <slug> --name ... --agent ...`

#### Feature shaping protocol

When an idea starts converging toward something buildable:
1. **Seed** — working name, one-liner, hypothesis, trigger
2. **User perspectives** — who uses this, who's affected, who's skeptical
3. **Expert consultation** — technical feasibility, architecture fit, constraints
4. **Shape** — core value prop, user stories, design decisions, trade-offs
5. **Define MVP** — minimum lovable, not minimum viable
6. **Step back** — do we need this? scope creep? quiet voices heard?

---

## Session Log Protocol

**Every session must be logged before ending.** The log is the persistent record
of the dialogue — it's how future sessions restore context without re-reading transcripts.

> ⚠️ **Sessions are not scrapbook artifacts — these are different things.**
>
> - ❌ Do NOT use the `Write` tool to create session files directly in the ponder directory
> - ❌ Do NOT use `sdlc ponder capture` to save sessions
> - ✓ ALWAYS use `sdlc ponder session log` — this is the only correct path
>
> Why it matters: `sdlc ponder session log` auto-numbers the file, places it in the
> correct `sessions/` subdirectory, increments the session counter in the manifest,
> and mirrors the orientation fields so future sessions and the web UI can read them.
> Skipping this command means the session is invisible — it becomes an artifact,
> not a session.

### Session file format

Session files are Markdown with a YAML frontmatter block. The frontmatter carries
metadata; the body is the free-form dialogue.

```markdown
---
session: <N>
timestamp: <ISO-8601 UTC>
orientation:
  current: "<one-liner: where the thinking is right now>"
  next: "<one-liner: concrete next action or focus>"
  commit: "<one-liner: condition that unlocks commitment>"
---

<session dialogue here — tool calls, partner voices, sketches, decisions, questions>
```

Inline markers to use consistently:
- `⚑  Decided:` — something resolved, with brief rationale
- `?  Open:` — unresolved question or tension still alive
- `Recruited: NAME · ROLE` — when a new partner joins mid-session
- `**NAME · ROLE**` — header for each partner voice block

### The only correct logging procedure

1. Write the session content to a temp file using the Write tool:
```bash
# Write tool → /tmp/ponder-session-<slug>.md
```
2. Register it:
```bash
sdlc ponder session log <slug> --file /tmp/ponder-session-<slug>.md
```

The system auto-assigns the session number — do not try to number the file yourself.

---

## Ending the session

Before summarizing:

1. **Compose the session document.** Write a complete Markdown file to
   `/tmp/ponder-session-<slug>.md` using the Write tool. Include everything that
   happened — tool calls, partner voices, sketches, decisions (⚑), open questions (?),
   and recruitment events. Set the orientation fields to reflect where the thinking
   is right now, what should happen next, and what unlocks commitment.

2. **Log it:**
```bash
sdlc ponder session log <slug> --file /tmp/ponder-session-<slug>.md
```

3. **Summarize** what was explored, what was captured, and what remains unexplored.
   Include the orientation strip so the user sees it.

Always end with **Next:**

| State | Next |
|---|---|
| Early exploration, many gaps | `**Next:** /sdlc-ponder <slug>` (continue exploring) |
| Direction emerging, need depth | `**Next:** /sdlc-ponder <slug>` (continue with focus on <gap>) |
| Idea shaped, ready to commit | `**Next:** /sdlc-ponder-commit <slug>` |
| Idea explored and parked | `**Next:** /sdlc-ponder` (explore something else) |
"#;

const SDLC_PONDER_PLAYBOOK: &str = r#"# sdlc-ponder

Open the ponder workspace for creative exploration and ideation.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. If a slug is provided, load the entry:
   - `sdlc ponder show <slug>`
   - `sdlc ponder session list <slug>` — if sessions exist, read the most recent one:
     `sdlc ponder session read <slug> <N>`
   - Orient from the orientation strip (WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL).
   - Summarize what's been explored, open questions, and what to do next.
2. If a new idea is provided, create the entry:
   - `sdlc ponder create <slug> --title "<title>"`
   - `sdlc ponder capture <slug> --content "<brief>" --as brief.md`
   - Recruit 2-3 thought partners: domain expert, end-user advocate, pragmatic skeptic.
3. If no arguments: `sdlc ponder list` and ask which to explore.
4. Facilitate: interrogate the brief, channel thought partners, suggest captures.
5. When artifacts are ready: `sdlc ponder capture <slug> --content "..." --as <name>.md`.
6. Before ending: write and log the session file:
   - Compose a Markdown session with YAML frontmatter (session, timestamp, orientation).
   - `sdlc ponder session log <slug> --file /tmp/session-<N>.md`
7. End with **Next:** — continue exploring, commit, or park.
"#;

const SDLC_PONDER_SKILL: &str = r#"---
name: sdlc-ponder
description: Open the ideation workspace for creative exploration. Use when exploring, interrogating, or developing ideas before they become features.
---

# SDLC Ponder Skill

Use this skill to open a ponder workspace for exploring ideas.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. If slug given: `sdlc ponder show <slug>`. Then `sdlc ponder session list <slug>` —
   read the most recent session to restore context. Orient from the orientation strip.
2. If new idea: create entry, capture brief, recruit 2-3 thought partners.
3. If no args: `sdlc ponder list`. Ask which to explore.
4. Facilitate: interrogate, channel partners, capture artifacts.
5. Capture with `sdlc ponder capture <slug> --content "..." --as <name>.md`.
6. Before ending: compose session Markdown with YAML frontmatter (session, timestamp,
   orientation) and log it: `sdlc ponder session log <slug> --file /tmp/session.md`.
7. End with **Next:** — continue, commit, or park.
"#;

pub static SDLC_PONDER: CommandDef = CommandDef {
    slug: "sdlc-ponder",
    claude_content: SDLC_PONDER_COMMAND,
    gemini_description: "Open the ideation workspace for exploring ideas with thought partners",
    playbook: SDLC_PONDER_PLAYBOOK,
    opencode_description: "Open the ideation workspace for exploring ideas with thought partners",
    opencode_hint: "[slug or new idea]",
    skill: SDLC_PONDER_SKILL,
};
