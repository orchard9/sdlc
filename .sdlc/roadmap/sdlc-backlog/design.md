# sdlc backlog — Design Document

## Problem Statement

During autonomous sdlc runs, agents identify out-of-scope concerns that have no natural home. Every current capture path is wrong: `sdlc task add` requires a feature slug, `sdlc comment --flag fyi` is feature-local and invisible to the next agent, `sdlc escalate` requires human action and stops the run, advisory findings are for macro health analysis not operational session notes. Result: concerns get lost.

## Decision: New Primitive (not advisory + guidance)

Advisory findings were evaluated as an alternative. Rejected because:
- Advisory lifecycle: health observation → ponder topic (macroscopic)
- Backlog lifecycle: session discovery → feature (operational)
- Advisory has no promotion path to a feature
- Mixing operational notes with health analysis degrades the advisory signal quality

## Data Model

```rust
// crates/sdlc-core/src/backlog.rs
pub struct BacklogItem {
    pub id: String,                     // B1, B2, B3...
    pub title: String,                  // Required. Short concern statement.
    pub description: Option<String>,    // Optional longer context.
    pub status: BacklogStatus,          // open | parked | promoted
    pub source_feature: Option<String>, // Provenance: which feature run produced this?
    pub promoted_to: Option<String>,    // Slug of the feature created from promotion
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum BacklogStatus { Open, Parked, Promoted }
```

Storage: `.sdlc/backlog.yaml` — flat list, same pattern as escalations.yaml

No kind, no tags, no priority in v1.

## CLI Surface

```
sdlc backlog add <title...>                                    # Required: title (variadic)
sdlc backlog add <title...> --description "..." --source-feature <slug>

sdlc backlog list                  # Open only (default)
sdlc backlog list --all            # All statuses
sdlc backlog list --status parked

sdlc backlog park <id>             # De-prioritize without deletion
sdlc backlog promote <id> --slug <feature-slug>   # Creates feature + sets promoted_to
sdlc backlog promote <id>          # Derives slug from title if not given
sdlc backlog show <id>             # Full detail
```

No delete command. Items persist as historical record.

## Promotion Semantics

`promote` calls `sdlc feature create <slug> --title "..."` internally, sets `promoted_to` on the backlog item. Feature begins at draft phase and follows the normal state machine. No artifacts written automatically.

## Files to Create/Modify

### sdlc-core
- CREATE: `crates/sdlc-core/src/backlog.rs` — BacklogItem struct, BacklogStatus enum, add/list/park/promote/get/load_all/save_all, unit tests
- MODIFY: `crates/sdlc-core/src/paths.rs` — add BACKLOG_FILE const + backlog_path() fn
- MODIFY: `crates/sdlc-core/src/lib.rs` — add pub mod backlog
- MODIFY: `crates/sdlc-core/src/error.rs` — add BacklogItemNotFound(String) variant

### sdlc-cli
- CREATE: `crates/sdlc-cli/src/cmd/backlog.rs` — BacklogSubcommand enum, run() fn
- MODIFY: `crates/sdlc-cli/src/cmd/mod.rs` — add pub mod backlog
- MODIFY: `crates/sdlc-cli/src/main.rs` — add Backlog subcommand to Commands enum + dispatch

### sdlc-server
- CREATE: `crates/sdlc-server/src/routes/backlog.rs` — list, create, park, promote handlers + route tests
- MODIFY: `crates/sdlc-server/src/routes/mod.rs` — add pub mod backlog
- MODIFY: `crates/sdlc-server/src/lib.rs` — register routes (GET /api/backlog, POST /api/backlog, POST /api/backlog/:id/park, POST /api/backlog/:id/promote)

### Frontend
- MODIFY: `frontend/src/pages/Dashboard.tsx` — add Backlog section (open items, promote/park inline)
- (Later) NEW: `frontend/src/pages/BacklogPage.tsx` + sidebar/nav entry

### Guidance & Agent Commands
- MODIFY: `.sdlc/guidance.md` §6 — add backlog commands to table
- MODIFY: `.sdlc/guidance.md` — add §12 Session Close Protocol
- MODIFY: `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` — add out-of-scope concerns instruction
- MODIFY: `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — same
- MODIFY: GUIDANCE_MD_CONTENT in init templates — add backlog commands to §6 table

## Guidance Wording (for sdlc-run / sdlc-next)

> **Discovered out-of-scope concerns:** If during this run you identify concerns that are real and important but cannot be addressed within the current feature's scope — architectural issues, cross-cutting debt, systemic problems — call `sdlc backlog add "title" --source-feature <slug>` for each one before the run ends. Do NOT skip this step. Concerns captured in the backlog surface in `sdlc next` and on the Dashboard, and can be promoted to features when the time is right. A concern left uncaptured is a concern lost.

## §12 Session Close Protocol (guidance.md)

When a run completes, before reporting done:

1. Review what was worked on. Did any out-of-scope concerns surface?
2. If yes: `sdlc backlog add "concern description" --source-feature <slug>` for each one.
3. Check: `sdlc backlog list` — confirm open items are visible.

| Action | Command |
|---|---|
| Add a backlog item | `sdlc backlog add <title...>` |
| Add with context | `sdlc backlog add <title...> --description "..." --source-feature <slug>` |
| List open items | `sdlc backlog list` |
| Park (de-prioritize) | `sdlc backlog park <id>` |
| Promote to feature | `sdlc backlog promote <id> --slug <feature-slug>` |

## Milestone Scoping

This is a single self-contained milestone: `v13-backlog` (or whichever v-number is next).

Wave 1: core + CLI (the value-delivering part)
- sdlc-core/backlog.rs
- sdlc-cli/backlog.rs
- guidance updates
- sdlc-run / sdlc-next command updates

Wave 2: server + dashboard (visibility)
- sdlc-server/routes/backlog.rs
- Dashboard.tsx Backlog section