# Root Cause Interface

## What it is

An investigation workspace that runs the root-cause skill protocol as structured agent phases. The user describes a broken thing; the interface runs a forensic investigation across five areas and produces either a fix task (urgently prioritized) or a guideline if the cause is systemic.

Behaves like Ponder — a session-based dialogue with a structured phase strip and artifact workspace — but with a rigid phase order, investigation perspective cards, and a typed output gate at the end.

---

## Phase Model

```
Triage → Investigate → Synthesize → Output
```

Each phase maps to the skill protocol:

| Phase | What happens | Agent does |
|-------|-------------|-----------|
| **Triage** | Establish symptom, reproduction, severity, what changed | Asks clarifying questions, fills triage report, writes `triage.md` to workspace |
| **Investigate** | Five parallel areas of analysis | Reads code, traces paths, writes one artifact per area |
| **Synthesize** | Root cause hypothesis with confidence ≥ 70% | Writes `synthesis.md` with proximate/root/confidence/competing |
| **Output** | Decide: fix task or guideline | Creates task or doc, marks investigation complete |

Phase advances when the agent writes the phase-gate artifact (triage.md → synthesis.md → output). No manual transitions. The phase strip updates via SSE exactly like the orientation strip does in Ponder.

---

## Perspectives (Investigation Areas)

During Investigate phase, five named perspectives are active simultaneously. Each perspective is a named view with its own status and finding:

| # | Name | What it looks at |
|---|------|-----------------|
| 1 | Code Paths | Entry point → callers → handlers → data layer |
| 2 | Bottlenecks | Timeouts, race conditions, N+1 queries, memory |
| 3 | Data Flow | State integrity, serialization, schema changes |
| 4 | Auth Chain | Token validation, permission checks, isolation |
| 5 | Environment | Config drift, env vars, deploys, dependency versions |

The UI shows these as five cards in the workspace panel (replacing the file browser during Investigation phase). Each card shows:
- Status: `pending` / `investigating` / `finding` / `hypothesis`
- One-line finding (populated by the agent)
- Confidence score once complete

The agent populates these by writing structured artifacts: `area-1-code-paths.md`, `area-2-bottlenecks.md`, etc. The server parses known filenames to extract status/confidence for the cards.

---

## UI Structure

Reuses the Ponder shell exactly:

```
┌──────────────────────────────────────────────────────────┐
│ ← Title                              [status] [📁]       │  ← Same header as Ponder
├──────────────────────────────────────────────────────────┤
│ [Triage] → [Investigate] → [Synthesize] → [Output]       │  ← Phase strip (new, replaces OrientationStrip)
├─────────────────────────────────────┬────────────────────┤
│                                     │                    │
│  Session dialogue stream            │  Workspace panel   │  ← Same split as Ponder
│                                     │  (area cards       │
│                                     │   during invest.;  │
│                                     │   artifact files   │
│                                     │   otherwise)       │
│                                     │                    │
├─────────────────────────────────────┴────────────────────┤
│  Add context / answer questions...                       │  ← Same input bar
└──────────────────────────────────────────────────────────┘
```

**What's reused from Ponder unchanged:**
- `WorkspacePanel` — artifact file browser with left/right navigation
- `FullscreenModal` — artifact fullscreen view
- `MarkdownContent` — markdown/code rendering
- `InputBar` — send message to agent
- Mobile bottom sheet pattern
- SSE-driven live updates
- List → detail split layout

**What's new or adapted:**
- `PhaseStrip` — horizontal stepper replacing `OrientationStrip`; reads phase from manifest
- `AreaCards` — five perspective cards shown in workspace during Investigate phase; replaces file list
- `OutputGate` — shown when phase = Output; two buttons: "Create Fix Task" / "Create Guideline"
- Investigation list page (mirrors PonderPage)

---

## Data Model

```yaml
# .sdlc/investigations/<slug>/manifest.yaml
slug: user-auth-null-pointer
title: "User auth null pointer on login"
kind: root_cause           # discriminator field (root_cause | evolve | guideline)
phase: investigate         # triage | investigate | synthesize | output | done
status: in_progress        # in_progress | complete | parked
context: "NullPointerException in AuthService.validate() for SSO users"
created_at: "..."
updated_at: "..."
confidence: null           # populated after synthesize phase (0-100)
output_type: null          # task | guideline (set during output phase)
output_ref: null           # feature slug or guideline path
```

All investigations share a flat directory — `kind` in the manifest discriminates the type. There is no type-prefix in the path.

Artifacts (same structure as ponder scrapbook):
```
.sdlc/investigations/<slug>/
  manifest.yaml
  sessions/
    session-001.md
    session-002.md
  triage.md
  area-1-code-paths.md
  area-2-bottlenecks.md
  area-3-data-flow.md
  area-4-auth-chain.md
  area-5-environment.md
  synthesis.md
```

---

## Output Types

### Fix Task
Creates a new feature entry with:
- Phase: `draft` (immediately active)
- Priority: urgent (surfaced at top of task queue)
- Title from synthesis root cause
- Description: link to investigation + synthesis excerpt
- Tag: `[investigation]`

```bash
sdlc feature create <derived-slug> --title "Fix: <root cause>" --priority urgent
# then attach the synthesis as the spec artifact
sdlc artifact draft <derived-slug> spec
```

### Guideline
Writes a markdown document to `.sdlc/guidelines/<slug>.md`. Format:

```markdown
# <Title>

**Problem:** What breaks when this guideline is ignored.

## Rules
1. ...
2. ...

## Evidence
- `path/to/file.go:123` — example of the violation
- Investigation: [<slug>](.sdlc/investigations/root-cause/<slug>/)

## Rationale
Why these rules prevent the root cause.
```

---

## Backend (CLI commands)

```bash
# Investigation lifecycle
sdlc investigate create <slug> --kind root-cause --title "..." --context "..."
sdlc investigate show <slug>
sdlc investigate list [--kind root-cause] [--status in_progress]
sdlc investigate update <slug> --phase <phase> --status <status> --confidence <0-100>
sdlc investigate artifacts <slug>

# Artifact capture
sdlc investigate capture <slug> --content "..." --as <filename>
sdlc investigate capture <slug> --file /path/to/file --as <filename>

# Session logging (MANDATORY two-step protocol — see below)
sdlc investigate session log <slug> --file /tmp/investigation-session-<slug>.md
sdlc investigate session list <slug>
sdlc investigate session read <slug> <N>
```

## Session Protocol

Agents MUST follow this exact two-step procedure to log sessions:

1. Write the complete session Markdown to a temp file:
   ```bash
   # Write tool → /tmp/investigation-session-<slug>.md
   ```
2. Run:
   ```bash
   sdlc investigate session log <slug> --file /tmp/investigation-session-<slug>.md
   ```

**NEVER do these — they create artifacts, not sessions:**
- ❌ Write tool directly to `.sdlc/investigations/<slug>/session-N.md`
- ❌ `sdlc investigate capture` with session content
- ❌ Any path other than the two-step Write → `sdlc investigate session log` flow

Why: `sdlc investigate session log` auto-numbers the file, places it in `sessions/`, increments the session counter in the manifest, and mirrors orientation. Bypassing it breaks the session count and leaves orientation stale.

---

## Agent Prompt / Skill

The investigation chat agent gets the root-cause skill embedded in its system prompt, plus context injection:

```
You are running a root cause investigation.
Current phase: {phase}
Context: {context}

[embed root-cause SKILL.md content]

Artifacts already written: {list of artifacts in workspace}
```

The agent reads, investigates, writes artifacts via `sdlc investigate capture`, advances phases by writing gate artifacts, and proposes output via the input bar.

---

## Implementation Order

1. **Data layer** — `InvestigationEntry` struct in `sdlc-core`, CLI commands (or reuse ponder with type field)
2. **Server routes** — `/api/investigations/*` (or extend roadmap routes with type filter)
3. **`PhaseStrip` component** — horizontal stepper, reads phase from manifest
4. **`AreaCards` component** — five perspective cards, reads area artifacts
5. **List + detail page** — mirrors PonderPage, routes to `/investigations/root-cause`
6. **`OutputGate` component** — shown in phase=output; creates task or guideline
7. **Agent prompt** — wire root-cause skill into investigation chat
