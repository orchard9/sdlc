# Design Shape: Spike Interface + Promote to Ponder

## Problem Being Solved

Spikes are invisible and disconnected from the rest of the workflow:
- Findings land in `.sdlc/spikes/<slug>/findings.md` — nothing reads this folder
- No UI, no REST routes, no Rust data layer for spikes
- The spike → ponder handoff is a text suggestion only, no actual mechanism
- After ADAPT/REJECT verdict, ponder starts blank — spike learnings aren't carried

## Core Components

### 1. Rust Data Layer (`sdlc-core/src/spikes.rs`)

Load-only, no decisions. Reads `.sdlc/spikes/<slug>/findings.md` and parses header:

```rust
pub struct SpikeEntry {
    pub slug: String,
    pub title: String,          // from findings.md "# Spike: <title>"
    pub verdict: Option<String>, // ADOPT | ADAPT | REJECT (from "**Verdict:** " line)
    pub date: Option<String>,   // from "**Date:** " line
    pub the_question: Option<String>, // extracted from "## The Question" section
}

pub fn list(root: &Path) -> Result<Vec<SpikeEntry>>
pub fn load_findings(root: &Path, slug: &str) -> Result<String>
pub fn promote_to_ponder(root: &Path, slug: &str, ponder_slug: Option<&str>) -> Result<String>
```

`promote_to_ponder` extracts sections from findings.md and seeds a new ponder:
- Creates ponder entry with title from `The Question`
- Captures full findings as `spike-findings.md`
- Extracts `## Risks and Open Questions` section → `open-questions.md`
- Returns the new ponder slug

### 2. CLI (`sdlc-cli`)

```
sdlc spike list
sdlc spike show <slug>
sdlc spike promote <slug> [--as <ponder-slug>]
```

`promote` prints the new ponder slug on stdout so the calling agent can immediately open it.

### 3. REST Routes (`sdlc-server/src/routes/spikes.rs`)

```
GET  /api/spikes              → list all spikes
GET  /api/spikes/:slug        → findings content + parsed metadata
POST /api/spikes/:slug/promote → { ponder_slug?: string } → creates ponder, returns { ponder_slug }
```

### 4. UI (`SpikePage.tsx`)

Three views (see mockup: spike-interface-mockup.html):

**List View** — Sidebar link in "Explore" group (alongside Ponder, Investigations)
- Verdict badge (ADOPT=green, ADAPT=yellow, REJECT=red, IN PROGRESS=gray)
- Need one-liner (truncated)
- Date
- ADAPT/REJECT rows show "promote to ponder →" affordance
- Summary stats at bottom (N total, N adopted, N adapting, N rejected)

**Detail View** — Click a spike → see findings rendered with structured sections
- Header: verdict badge + date
- "Promote to Ponder →" button (hidden for ADOPT — those go to hypothetical-planning)
- Findings card: The Question / Success Criteria / Candidates table / Risks and Open Questions
- File path hint in card header

**Promote View** — Confirmation + what gets carried forward
- List: spike-findings.md ✓ / open-questions.md ✓ / Ponder title ✓ / Prototype code — (ephemeral)
- Editable ponder slug (default: same as spike slug)
- Primary action: "Promote to Ponder →"
- CLI equivalent hint: `sdlc spike promote <slug>`
- On success: navigate to new ponder

## Key Design Decisions

⚑ **ADOPT spikes don't show "Promote to Ponder"** — ADOPT verdict means the thing is being built, not explored. The next step is `/sdlc-hypothetical-planning`, not pondering.

⚑ **Prototype code is not carried forward** — ephemeral, stays in /tmp. If worth preserving, the spike author should have run `sdlc spike preserve` (future) or committed it manually.

⚑ **"Risks and Open Questions" is the ponder's fuel** — not the full findings. Mara's insight: the ponder needs to start from what the spike couldn't resolve, not what it validated.

? **Should REJECT also promote?** — Or should REJECT findings be archived/closed? REJECT means the approach is dead, but the *need* might still be alive. Lean: yes, REJECT can promote — the ponder explores alternative approaches.

? **Does spike need sessions/chat?** — Current model: spikes are agent-only (Claude Code runs /sdlc-spike). No chat interface needed. The UI is read-only + promote action. Revisit if we want web-triggered spike runs.

## What's Not in V1

- Triggering a spike run from the UI (runs via /sdlc-spike CLI only)
- Spike sessions/chat history
- `sdlc spike preserve` for saving prototype code
- ADOPT pathway ("Convert to Feature Plan" button)
