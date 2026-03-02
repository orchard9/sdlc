use crate::cmd::init::registry::CommandDef;

const SDLC_CONVO_MINE_COMMAND: &str = r#"---
description: Mine conversations for actionable signal — analyze casual team chats, group by theme, and launch parallel ponder sessions for each group.
argument-hint: [file path or paste conversation text]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task, AskUserQuestion
---

# sdlc-convo-mine

Transform raw conversation dumps — Slack exports, Discord threads, meeting notes, casual team chats — into structured ideation. Read the conversations through multiple lenses, extract signal, group themes, and launch a ponder workspace for each group.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Input

**$ARGUMENTS** is one of:
- A file path to a conversation dump (`.txt`, `.md`, `.json`, etc.)
- Inline conversation text pasted directly
- Empty — ask the user to paste or provide a path

If a file path is given, read it with the Read tool. If inline text, use it directly. If empty, use AskUserQuestion to prompt the user.

---

## Phase 1: Multi-Perspective Analysis

Read the entire conversation through five lenses. For each lens, extract items that appear significant, recurring, or emotionally charged. Do not filter yet — capture everything that registers.

### Product/User Lens
What are people struggling with as users of the product? Repeated complaints, feature gaps, confusing UX, missing workflows, things that "should just work". What do real users want that they're not getting?

### Engineering/Technical Lens
Technical debt mentions, architecture concerns, scaling fears, reliability frustrations, "we should really fix...", "this is getting hairy", tooling pain, test gaps, performance issues, fragility.

### Process/Workflow Lens
Team coordination friction, "we keep doing this manually", unclear ownership, repeated miscommunications, painful handoffs, missing automation, ambiguous responsibilities, things that fall through the cracks.

### Strategy/Business Lens
Competitive observations, market opportunities, user growth signals, revenue ideas, "what if we...", strategic risks, things competitors are doing, partnership ideas, positioning concerns.

### Culture/Team Lens
Morale signals, trust issues, collaboration breakdown, "I feel like nobody reads...", onboarding friction, knowledge silos, recognition gaps, team energy and motivation indicators.

---

## Phase 2: Signal Extraction and Evaluation

For each item extracted in Phase 1, evaluate three dimensions:

1. **Frequency** — mentioned once, or recurring theme across the conversation?
2. **Emotion** — flat observation vs. genuine frustration, excitement, or concern?
3. **Actionability** — is there a concrete thing that could be built, fixed, or changed?

Also check existing ponders for overlap:
```bash
sdlc ponder list
```

Assign each item a signal strength:
- **Strong** — recurring + emotional + actionable (or: new + high-stakes + concrete)
- **Weak** — mentioned once, passing reference, or unclear what to do with it
- **Noise** — jokes, pleasantries, off-topic tangents → drop entirely

---

## Phase 3: Thematic Grouping

Cluster strong-signal items into thematic groups. A good group:
- Has 2+ related items
- Addresses a coherent problem or opportunity space
- Has a crisp, noun-phrase slug and title

Name groups by what they're about, not who mentioned them. Examples:
- `onboarding-friction` — "First week is overwhelming for new users"
- `deploy-confidence` — "Deploys are scary, no rollback, team avoids them"
- `search-experience` — "Search fails power users with complex queries"

Each group needs:
- **slug** — lowercase, hyphens, ≤ 40 chars
- **title** — concise noun phrase (4-6 words)
- **summary** — 1-2 sentences: what the group is about and why it matters
- **key signals** — 2-5 specific extracted items
- **relevant excerpts** — the actual conversation lines, verbatim, that surfaced this

If ≥ 3 weak signals exist, create a `signal-watch` group as a catch-all parking lot.

Show the user the proposed groups before creating ponders:

```
Proposed groups:
  1. onboarding-friction  — "First week is overwhelming for new users" (4 strong signals)
  2. deploy-confidence    — "Deploys feel dangerous, team avoids pushing" (3 strong signals)
  3. search-experience    — "Search doesn't work for complex queries" (2 strong signals)
  4. signal-watch         — Weak signals to revisit later (5 weak items)
```

---

## Phase 4: Ponder Creation

For each group, create a ponder and pre-load it with context.

### Create the entry
```bash
sdlc ponder create <slug> --title "<title>"
```

### Capture a brief

Write the brief to a temp file first, then capture:
```bash
# Write tool → /tmp/convo-mine-<slug>-brief.md
sdlc ponder capture <slug> --file /tmp/convo-mine-<slug>-brief.md --as brief.md
```

The brief should contain:
- **Origin** — "Extracted from conversation dump on `<date>`"
- **Summary** — what this group is about and why it matters
- **Key signals** — bullet list of extracted items with signal strength and lens
- **Relevant excerpts** — the actual conversation lines that surfaced this, verbatim
- **Open questions** — what needs to be understood before this can become a feature or change

### Log an initial session

Write the session file:
```bash
# Write tool → /tmp/ponder-session-<slug>.md
```

Session format:
```markdown
---
session: 1
timestamp: <ISO-8601 UTC>
orientation:
  current: "Raw signal from conversation — not yet shaped into a problem statement"
  next: "Interrogate the brief: is there a real, concrete problem worth acting on?"
  commit: "Clear problem statement + at least one user story + rough direction"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a conversation dump.

### Signals extracted

<key signals and excerpts from the group>

### Why this might matter

<synthesized reasoning: what does this signal suggest about a real need or gap?>

### Open questions

<what needs to be answered before this becomes actionable?>

### Suggested first exploration

<what the first /sdlc-ponder session should focus on — a specific angle, question, or user to interrogate>
```

Register the session:
```bash
sdlc ponder session log <slug> --file /tmp/ponder-session-<slug>.md
```

Repeat for every group.

---

## Phase 5: Summary and Launch

Output a table of all created ponders:

| Ponder | Title | Signals | Launch |
|--------|-------|---------|--------|
| `onboarding-friction` | First Week Friction | 4 strong | `/sdlc-ponder onboarding-friction` |
| `deploy-confidence` | Deploy Confidence | 3 strong | `/sdlc-ponder deploy-confidence` |
| `signal-watch` | Signal Watch | 5 weak | `/sdlc-ponder signal-watch` |

Always end with **Next:**

| State | Next |
|---|---|
| One obvious high-priority group | `**Next:** /sdlc-ponder <highest-priority-slug>` |
| Multiple groups worth parallel exploration | `**Next:** /do-parallel` — one `/sdlc-ponder` per group |
| Only weak signals found | `**Next:** /sdlc-ponder signal-watch` — explore together |
| No actionable signal found | `**Next:** Ask user for more context or a different conversation dump |
"#;

const SDLC_CONVO_MINE_PLAYBOOK: &str = r#"# sdlc-convo-mine

Mine conversation dumps for actionable signal and launch parallel ponder sessions.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Read the conversation from $ARGUMENTS file path, inline text, or prompt the user.
2. Apply 5 lenses: Product/User · Engineering/Technical · Process/Workflow · Strategy/Business · Culture/Team.
3. For each extracted item, rate signal strength (strong/weak) based on frequency, emotion, and actionability.
4. Check existing ponders: `sdlc ponder list` — avoid duplicating active ideas.
5. Group strong signals by theme — each group needs: slug, title (noun phrase), summary, key signals, excerpts.
6. Aggregate weak signals (≥ 3) into a `signal-watch` catch-all group.
7. Show proposed groups before creating ponders.
8. For each group:
   - `sdlc ponder create <slug> --title "<title>"`
   - Write brief.md (origin, summary, key signals, excerpts, open questions) → `sdlc ponder capture <slug> --file /tmp/brief.md --as brief.md`
   - Write initial session (orientation: raw signal → interrogate → problem statement) → `sdlc ponder session log <slug> --file /tmp/session.md`
9. Output summary table with `/sdlc-ponder <slug>` launch commands per group.
10. End with **Next:** — single group, or `/do-parallel` for multiple.
"#;

const SDLC_CONVO_MINE_SKILL: &str = r#"---
name: sdlc-convo-mine
description: Mine conversation dumps for actionable signal. Apply 5 perspective lenses, group themes, launch ponder sessions per group. Use when analyzing Slack exports, meeting notes, or casual team chats.
---

# SDLC Convo-Mine Skill

Transform raw conversation text (Slack, Discord, meeting notes, team chats) into structured ideation workspaces.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Read conversation from file path or inline text.
2. Apply 5 lenses: Product/User · Engineering · Process · Strategy · Culture.
3. Extract items per lens. Rate each: strong (recurring + emotional + actionable) or weak or noise.
4. Check `sdlc ponder list` for existing overlap.
5. Group strong signals by theme — slug, title, summary, key signals, excerpts.
6. Collect ≥ 3 weak signals into a `signal-watch` parking lot group.
7. Show proposed groups to user.
8. For each group: `sdlc ponder create`, capture brief.md with excerpts, log initial session with orientation.
9. Output summary table with launch commands.
10. End: `/sdlc-ponder <slug>` for single group, or `/do-parallel` for multiple.
"#;

pub static SDLC_CONVO_MINE: CommandDef = CommandDef {
    slug: "sdlc-convo-mine",
    claude_content: SDLC_CONVO_MINE_COMMAND,
    gemini_description:
        "Mine conversation dumps for actionable signal and launch ponder sessions per theme group",
    playbook: SDLC_CONVO_MINE_PLAYBOOK,
    opencode_description:
        "Mine conversation dumps for actionable signal and launch ponder sessions per theme group",
    opencode_hint: "[file path or conversation text]",
    skill: SDLC_CONVO_MINE_SKILL,
};
