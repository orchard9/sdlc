# sdlc backlog — Commit Plan

## Problem

During autonomous sdlc runs agents discover out-of-scope concerns — architectural issues, cross-cutting debt, systemic observations — that have no capture path. Every existing mechanism is wrong: `sdlc task add` requires a feature slug, `sdlc comment --flag fyi` is feature-local and invisible to the next session, `sdlc escalate` stops the run and requires human action. Result: concerns get lost between sessions.

## Solution

New primitive: `sdlc backlog` — a project-level parking lot for out-of-scope concerns with a clear promotion path to features.

## Milestone

**v13-backlog** — "Project-level backlog: capture and promote out-of-scope concerns"

Vision: An agent finishing a run calls `sdlc backlog add "..."` for every concern that surfaced but was out of scope. Those items are visible on the Dashboard. When it's time to act on one, a human or agent calls `sdlc backlog promote <id>` and a feature enters the state machine. Nothing is lost between sessions.

## Wave Plan

### Wave 1 — Core value (core + CLI + guidance)

**backlog-core** — Data model and storage in sdlc-core
- CREATE `crates/sdlc-core/src/backlog.rs` — BacklogItem struct (id: B1/B2..., title, description, status: open|parked|promoted, source_feature, promoted_to, created_at, updated_at), BacklogStatus enum, add/list/park/promote/get/load_all/save_all, unit tests
- MODIFY `crates/sdlc-core/src/paths.rs` — add BACKLOG_FILE const + backlog_path()
- MODIFY `crates/sdlc-core/src/lib.rs` — add pub mod backlog
- MODIFY `crates/sdlc-core/src/error.rs` — add BacklogItemNotFound(String) variant

**backlog-cli** — CLI surface in sdlc-cli
- CREATE `crates/sdlc-cli/src/cmd/backlog.rs` — BacklogSubcommand enum (Add, List, Park, Promote, Show), run() fn
- MODIFY `crates/sdlc-cli/src/cmd/mod.rs` — add pub mod backlog
- MODIFY `crates/sdlc-cli/src/main.rs` — add Backlog subcommand to Commands enum + dispatch
- Commands: `sdlc backlog add <title...> [--description "..."] [--source-feature <slug>]`, `sdlc backlog list [--all | --status <s>]`, `sdlc backlog park <id>`, `sdlc backlog promote <id> [--slug <feature-slug>]`, `sdlc backlog show <id>`

**backlog-guidance** — Guidance and agent command updates
- MODIFY `.sdlc/guidance.md` §6 — add backlog commands to CLI table
- MODIFY `.sdlc/guidance.md` — add §12 Session Close Protocol (review for concerns, call `sdlc backlog add` for each)
- MODIFY `GUIDANCE_MD_CONTENT` in `crates/sdlc-cli/src/cmd/init.rs` — add backlog commands to §6 table
- MODIFY sdlc-run command template — add "Discovered out-of-scope concerns" instruction before run ends
- MODIFY sdlc-next command template — same instruction

### Wave 2 — Visibility (server + dashboard)

**backlog-server** — REST API in sdlc-server
- CREATE `crates/sdlc-server/src/routes/backlog.rs` — list, create, park, promote handlers + route tests
- MODIFY `crates/sdlc-server/src/routes/mod.rs` — add pub mod backlog
- MODIFY `crates/sdlc-server/src/lib.rs` — register routes: GET /api/backlog, POST /api/backlog, POST /api/backlog/:id/park, POST /api/backlog/:id/promote

**backlog-dashboard** — Frontend visibility in Dashboard.tsx
- MODIFY `frontend/src/pages/Dashboard.tsx` — add Backlog section showing open items count + list, with inline park/promote actions

## Data Model

```rust
pub struct BacklogItem {
    pub id: String,                     // B1, B2, B3...
    pub title: String,
    pub description: Option<String>,
    pub status: BacklogStatus,          // Open | Parked | Promoted
    pub source_feature: Option<String>,
    pub promoted_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Storage: `.sdlc/backlog.yaml` — flat list, append pattern.

## Promotion Semantics

`promote` calls `sdlc feature create <slug>` internally, sets `promoted_to` on the backlog item. Feature enters draft phase. No artifacts auto-written.

## Session Close Guidance (verbatim for templates)

> **Discovered out-of-scope concerns:** If during this run you identified concerns that are real and important but cannot be addressed within the current feature's scope — architectural issues, cross-cutting debt, systemic problems — call `sdlc backlog add "<title>" --source-feature <slug>` for each one before the run ends. Do NOT skip this step. A concern left uncaptured is a concern lost.
