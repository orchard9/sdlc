---
session: 1
timestamp: 2026-03-02T02:30:00Z
orientation:
  current: "Full design decided — data model, CLI, all file touchpoints, guidance wording"
  next: "Commit to milestones/features via /sdlc-ponder-commit sdlc-backlog"
  commit: "Design is complete and agreed by all partners. Ready to commit."
---

## Problem Statement

During autonomous sdlc runs, agents identify out-of-scope concerns — architectural issues, cross-cutting debt — that have no natural home. Current capture paths are all wrong: `task add` requires a feature slug, `comment --flag fyi` is feature-local and invisible across sessions, `escalate` requires human action and stops the run, advisory findings are for macro health analysis not operational notes.

## Advisory as Alternative — Rejected

⚑  Decided: Advisory findings are the wrong home. The lifecycle is wrong (health observation → ponder, not session discovery → feature), there's no promotion path, and mixing operational notes with health analysis degrades advisory signal quality.

**Dan Reeves · Systems Minimalist** was the key challenger here. He accepted the justification once the promotion path distinction was made clear — "if the job could be done with existing tools plus a guidance change, I'd say don't build it. But advisory has no promotion path to a feature. That's a real gap."

## Data Model

**Felix Wagner · Tooling Architect** proposed:

```rust
pub struct BacklogItem {
    pub id: String,                     // B1, B2, B3
    pub title: String,                  // Required
    pub description: Option<String>,    // Optional
    pub status: BacklogStatus,          // open | parked | promoted
    pub source_feature: Option<String>, // Provenance — which run found this?
    pub promoted_to: Option<String>,    // Feature slug after promotion
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

⚑  Decided: No kind, no tags, no priority in v1. Dan enforced this hard and all agreed.

⚑  Decided: No delete. Items park or promote. Historical record preserved.

⚑  Decided: Sequential IDs (B1, B2...) matching escalation pattern (E1, E2...).

## CLI Surface

```
sdlc backlog add <title...>
sdlc backlog add <title...> --description "..." --source-feature <slug>
sdlc backlog list              (open only by default)
sdlc backlog list --all
sdlc backlog list --status parked
sdlc backlog park <id>
sdlc backlog promote <id> --slug <feature-slug>
sdlc backlog promote <id>     (derives slug from title)
sdlc backlog show <id>
```

⚑  Decided: Variadic title positional args matching `sdlc task add` pattern. Aria Chen insisted on this — "ergonomic consistency with existing commands is non-negotiable for agent adoption."

## Promotion Semantics

⚑  Decided: `promote` = `sdlc feature create <slug>` internally + set `promoted_to`. Feature starts at draft phase, follows normal state machine. No artifacts written automatically.

## Full File Touchpoint Map

Confirmed from deep codebase read:

### sdlc-core
- NEW `crates/sdlc-core/src/backlog.rs`
- MODIFY `crates/sdlc-core/src/paths.rs` — BACKLOG_FILE const, backlog_path()
- MODIFY `crates/sdlc-core/src/lib.rs` — pub mod backlog
- MODIFY `crates/sdlc-core/src/error.rs` — BacklogItemNotFound(String)

### sdlc-cli
- NEW `crates/sdlc-cli/src/cmd/backlog.rs`
- MODIFY `crates/sdlc-cli/src/cmd/mod.rs`
- MODIFY `crates/sdlc-cli/src/main.rs`

### sdlc-server
- NEW `crates/sdlc-server/src/routes/backlog.rs`
- MODIFY `crates/sdlc-server/src/routes/mod.rs`
- MODIFY `crates/sdlc-server/src/lib.rs`

### Frontend
- MODIFY `frontend/src/pages/Dashboard.tsx`
- (Later) NEW `frontend/src/pages/BacklogPage.tsx`

### Guidance & Commands
- MODIFY `.sdlc/guidance.md` §6 + add §12
- MODIFY `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs`
- MODIFY `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`
- MODIFY GUIDANCE_MD_CONTENT in init templates

## Guidance Wording (agreed)

For sdlc-run and sdlc-next "On discovery of out-of-scope concerns":
> If during this run you identify concerns that are real and important but cannot be addressed within the current feature's scope — architectural issues, cross-cutting debt, systemic problems — call `sdlc backlog add "title" --source-feature <slug>` for each one before the run ends. Do NOT skip this step. Concerns captured in the backlog surface in `sdlc next` and on the Dashboard, and can be promoted to features when the time is right. A concern left uncaptured is a concern lost.

**Aria Chen · Agent Ergonomics**: "The 'DO NOT skip this step' wording matters. Agents respond to explicit prohibitive instructions better than positive suggestions."

## Milestone Scoping

Wave 1 (CLI + guidance — delivers the core value immediately):
- sdlc-core/backlog.rs + paths/lib/error
- sdlc-cli/backlog.rs + main.rs registration
- guidance.md updates
- sdlc-run / sdlc-next command updates

Wave 2 (server + visibility):
- sdlc-server/routes/backlog.rs
- Dashboard.tsx Backlog section

## Open Questions

? Should `sdlc next` (project-level, no slug) surface backlog items alongside feature directives? Aria thinks yes — "the natural 'what should I work on' view." Not blocking v1 but worth deciding in the feature spec.

? Should `sdlc backlog promote` without `--slug` derive from title automatically, or ask? Current decision: derive deterministically (title → lowercase-kebab, truncated to 40 chars). No interactive prompts — agents can't interact.

## Recruited Team

- Felix Wagner · Developer tooling architect
- Aria Chen · AI agent ergonomics researcher
- Dan Reeves · Systems minimalist & skeptic
