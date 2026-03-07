# Standard Agents Spec: SDLC-Standard Agents for All Projects

## Decision

Every sdlc project gets a set of **standard agents** created during `sdlc init` scaffolding — not by the AI during specialize, but by the Rust CLI as template files. These agents power sdlc's own features (beat, knowledge, suggest) and exist regardless of project domain.

## Standard Agent Roster

### 1. Knowledge Librarian (`knowledge-librarian.md`)
- **Purpose:** Curates `.sdlc/knowledge/` — classifies entries, fills summaries, cross-references, publishes
- **Used by:** `/sdlc-knowledge`, knowledge maintenance workflows
- **Model:** `claude-sonnet-4-6` (classification/curation, not heavy reasoning)
- **Already exists** in this project — extract to template

### 2. CTO/CPO Lens (`cto-cpo-lens.md`)
- **Purpose:** Evaluates product direction against vision. Identifies drift from strategic objectives.
- **Used by:** `/sdlc-beat` (beat tool recruits this agent), advisory system
- **Model:** `claude-sonnet-4-6` (structured evaluation)
- **Already exists** in this project — extract to template

### 3. (Considered and rejected — see session notes)

## Where They Get Created

**`sdlc init` (Rust CLI)** — between step 6 (AGENTS.md) and step 7 (user scaffolding):
- Write `.claude/agents/knowledge-librarian.md` via `write_if_missing`
- Write `.claude/agents/cto-cpo-lens.md` via `write_if_missing`
- Use `write_if_missing` so specialize/recruit never clobbers manual edits

**NOT during specialize** — standard agents are infrastructure, not design decisions. Specialize creates project-specific agents on top.

## Template Content

Templates are embedded as `const` strings in `init/mod.rs` (or a new `init/standard_agents.rs`). Content matches the existing agent definitions but generalized:
- Remove project-specific knowledge catalog from librarian (use a generic starter)
- Keep cto-cpo-lens generic (it already is)

## Key Decisions
- `write_if_missing` not `atomic_write` — never overwrite user edits
- Standard agents go in `.claude/agents/` (project-level, shared via git)
- `sdlc update` also refreshes standard agents (same `write_if_missing` — only creates if absent)
- Specialize instructions updated: "You will see existing standard agents (knowledge-librarian, cto-cpo-lens). Do not replace them. Design project-specific agents to complement them."
